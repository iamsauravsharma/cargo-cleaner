use std::{io::Write, path::Path};

use anyhow::{Context, Result};
use colored::Colorize;
use structopt::{clap::AppSettings, StructOpt};

use crate::{
    crate_detail::CrateDetail,
    dir_path::DirPath,
    list_crate::CrateList,
    registry_dir::RegistryDir,
    utils::{convert_pretty, get_size, print_dash, query_print, show_top_number_crates},
};

#[derive(Debug, StructOpt)]
#[structopt(about="Perform operation only to registry related cache file", settings=&[
    AppSettings::ArgRequiredElseHelp,
])]
pub(crate) struct Registry {
    #[structopt(long = "all", short = "a", help = "Clean up all registry crates")]
    all: bool,
    #[structopt(
        long = "dry-run",
        short = "n",
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
    #[structopt(
        long = "light",
        short = "l",
        help = "Light cleanup repo by removing registry source but stores registry archive for \
                future compilation"
    )]
    light_cleanup: bool,
    #[structopt(long = "old", short = "o", help = "Clean old registry cache crates")]
    old: bool,
    #[structopt(
        long = "old-orphan",
        short = "z",
        help = "Clean registry crates which is both old and orphan"
    )]
    old_orphan: bool,
    #[structopt(
        long = "orphan",
        short = "x",
        help = "Clean orphan cache registry crates i.e all crates which are not present in lock \
                file generated till now use cargo trim -u to guarantee your all project generate \
                lock file"
    )]
    orphan: bool,
    #[structopt(
        long = "query",
        short = "q",
        help = "Return size of different .cargo/registry cache folders"
    )]
    query: bool,
    #[structopt(
        long = "remove",
        short = "r",
        help = "Remove provided crates from registry",
        value_name = "crate"
    )]
    remove: Option<Vec<String>>,
    #[structopt(
        long = "top",
        short = "t",
        help = "Show certain number of top crates which have highest size",
        value_name = "number"
    )]
    top: Option<usize>,
}

impl Registry {
    pub(super) fn run(
        &self,
        dir_path: &DirPath,
        crate_list: &CrateList,
        crate_detail: &CrateDetail,
        registry_crates_location: &mut RegistryDir,
        directory_is_empty: bool,
    ) -> Result<()> {
        let dry_run = self.dry_run;
        if self.light_cleanup {
            let light_cleanup_success =
                light_cleanup_registry(dir_path.src_dir(), dir_path.index_dir(), dry_run);
            if !light_cleanup_success {
                println!("Failed to delete some folder during light cleanup");
            }
        }
        if let Some(number) = self.top {
            top_crates_registry(crate_detail, number);
        }
        if self.query {
            let final_size = query_size_registry(dir_path, crate_list, crate_detail);
            query_print("Total size", &convert_pretty(final_size));
        }

        if self.old {
            let (sized_cleaned, total_crate_removed) =
                old_clean_registry(registry_crates_location, crate_list, crate_detail, dry_run);
            println!(
                "{}",
                format!(
                    "{} old crates removed which had occupied {:.3} MB",
                    total_crate_removed, sized_cleaned
                )
                .color("blue")
            );
        }

        if self.old_orphan {
            if directory_is_empty {
                let warning_text = "WARNING: You have not initialized any directory as rust \
                                    project directory. This command will clean all old crates \
                                    even if they are not orphan crates. Run command 'cargo trim \
                                    init' to initialize current directory as rust project \
                                    directory or pass cargo trim -d <directory> for setting rust \
                                    project directory";
                println!("{}", warning_text.color("yellow"));
                let mut input = String::new();
                print!("Do you want to continue? (y/N) ");
                std::io::stdout()
                    .flush()
                    .context("failed to flush output stream")?;
                std::io::stdin()
                    .read_line(&mut input)
                    .context("error: unable to read user input")?;
                let input = input.trim().to_ascii_lowercase();
                // if answer is any instead of yes and y return
                if !["y", "yes"].contains(&input.as_str()) {
                    return Ok(());
                }
            }
            let (sized_cleaned, total_crate_removed) = old_orphan_clean_registry(
                registry_crates_location,
                crate_list,
                crate_detail,
                dry_run,
            );

            println!(
                "{}",
                format!(
                    "{} crates which are both old and orphan crate removed which had {:.3} MB",
                    total_crate_removed, sized_cleaned
                )
                .color("blue")
            );
        }

        if self.orphan {
            if directory_is_empty {
                let warning_text = "WARNING: You have not initialized any directory as rust \
                                    project directory. This command will clean all crates since \
                                    all crates are classified as orphan crate. Run command 'cargo \
                                    trim init' to initialize current directory as rust project \
                                    directory or pass cargo trim -d <directory> for setting rust \
                                    project directory";
                println!("{}", warning_text.color("yellow"));
                let mut input = String::new();
                print!("Do you want to continue? (y/N) ");
                std::io::stdout()
                    .flush()
                    .context("failed to flush output stream")?;
                std::io::stdin()
                    .read_line(&mut input)
                    .context("error: unable to read user input")?;
                let input = input.trim().to_ascii_lowercase();
                // If answer is not y or yes then return
                if !["y", "yes"].contains(&input.as_str()) {
                    return Ok(());
                }
            }
            let (sized_cleaned, total_crate_removed) =
                orphan_clean_registry(registry_crates_location, crate_list, crate_detail, dry_run);

            println!(
                "{}",
                format!(
                    "{} orphan crates removed which had occupied {:.3} MB",
                    total_crate_removed, sized_cleaned
                )
                .color("blue")
            );
        }

        if self.all {
            let (sized_cleaned, total_crate_removed) =
                all_clean_registry(registry_crates_location, crate_list, crate_detail, dry_run);
            println!(
                "{}",
                format!(
                    "Total size of  {} crates removed :- {:.3} MB",
                    total_crate_removed, sized_cleaned
                )
                .color("blue")
            );
        }

        if let Some(crates) = &self.remove {
            remove_crates(
                crates,
                crate_list,
                registry_crates_location,
                crate_detail,
                dry_run,
            );
        }

        Ok(())
    }
}

// Perform light cleanup of registry and return if light clean was success or
// not
pub(super) fn light_cleanup_registry(src_dir: &Path, index_dir: &Path, dry_run: bool) -> bool {
    let mut light_cleanup_success = true;
    // delete src dir
    light_cleanup_success =
        crate::utils::delete_folder(src_dir, dry_run).is_ok() && light_cleanup_success;
    // Delete out .cache folder also
    light_cleanup_success =
        crate::utils::delete_index_cache(index_dir, dry_run).is_ok() && light_cleanup_success;
    light_cleanup_success
}

// Show top registry crates
pub(super) fn top_crates_registry(crate_detail: &CrateDetail, number: usize) {
    show_top_number_crates(
        crate_detail.registry_crates_archive(),
        "registry_archive",
        number,
    );
    show_top_number_crates(
        crate_detail.registry_crates_source(),
        "registry_source",
        number,
    );
}

// Query size of registry
pub(super) fn query_size_registry(
    dir_path: &DirPath,
    crate_list: &CrateList,
    crate_detail: &CrateDetail,
) -> u64 {
    let registry_dir_size = get_size(dir_path.registry_dir()).unwrap_or(0);
    query_print(
        &format!(
            "Total size of {} .cargo/registry crates:",
            crate_list.installed_registry().len()
        ),
        &convert_pretty(registry_dir_size),
    );
    query_print(
        &format!(
            "   \u{251c} Size of {} .cargo/registry/cache folder",
            crate_detail.registry_crates_archive().len()
        ),
        &convert_pretty(get_size(dir_path.cache_dir()).unwrap_or(0_u64)),
    );
    query_print(
        "   \u{251c} Size of .cargo/registry/index folder",
        &convert_pretty(get_size(dir_path.index_dir()).unwrap_or(0_u64)),
    );
    query_print(
        &format!(
            "   \u{2514} Size of {} .cargo/registry/src folder",
            crate_detail.registry_crates_source().len()
        ),
        &convert_pretty(get_size(dir_path.src_dir()).unwrap_or(0_u64)),
    );
    print_dash(crate::utils::query_full_width());
    registry_dir_size
}

// perform old clean on registry crates
pub(super) fn old_clean_registry(
    registry_crates_location: &mut RegistryDir,
    crate_list: &CrateList,
    crate_detail: &CrateDetail,
    dry_run: bool,
) -> (f64, usize) {
    (
        registry_crates_location.remove_crate_list(
            crate_detail,
            crate_list.old_registry(),
            dry_run,
        ),
        crate_list.old_registry().len(),
    )
}

// perform old orphan clean on registry crates
pub(super) fn old_orphan_clean_registry(
    registry_crates_location: &mut RegistryDir,
    crate_list: &CrateList,
    crate_detail: &CrateDetail,
    dry_run: bool,
) -> (f64, usize) {
    (
        registry_crates_location.remove_crate_list(
            crate_detail,
            &crate_list.list_old_orphan_registry(),
            dry_run,
        ),
        crate_list.list_old_orphan_registry().len(),
    )
}

// perform orphan clean on registry crates
pub(super) fn orphan_clean_registry(
    registry_crates_location: &mut RegistryDir,
    crate_list: &CrateList,
    crate_detail: &CrateDetail,
    dry_run: bool,
) -> (f64, usize) {
    (
        registry_crates_location.remove_crate_list(
            crate_detail,
            crate_list.orphan_registry(),
            dry_run,
        ),
        crate_list.orphan_registry().len(),
    )
}

// perform all install clean on registry crates
pub(super) fn all_clean_registry(
    registry_crates_location: &mut RegistryDir,
    crate_list: &CrateList,
    crate_detail: &CrateDetail,
    dry_run: bool,
) -> (f64, usize) {
    (
        registry_crates_location.remove_crate_list(
            crate_detail,
            crate_list.installed_registry(),
            dry_run,
        ),
        crate_list.installed_registry().len(),
    )
}

// Remove certain registry crates
fn remove_crates(
    crates: &[String],
    crate_list: &CrateList,
    registry_crates_location: &mut RegistryDir,
    crate_detail: &CrateDetail,
    dry_run: bool,
) {
    let mut size_cleaned = 0.0;
    for crate_name in crates {
        if crate_list.installed_registry().contains(crate_name) {
            registry_crates_location.remove_crate(crate_name, dry_run);
            size_cleaned += crate_detail.find_size_registry_all(crate_name);
        }
    }
    println!(
        "{}",
        format!("Total size removed :- {:.3} MB", size_cleaned).color("blue")
    );
}
