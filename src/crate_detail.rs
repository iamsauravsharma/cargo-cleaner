use std::collections::HashMap;

// stores different crate size and name information
#[derive(Default)]
pub(crate) struct CrateDetail {
    bin: HashMap<String, u64>,
    git_crates_source: HashMap<String, u64>,
    registry_crates_source: HashMap<String, u64>,
    git_crates_archive: HashMap<String, u64>,
    registry_crates_archive: HashMap<String, u64>,
}

impl CrateDetail {
    // return bin crates size information
    pub(crate) fn bin(&self) -> &HashMap<String, u64> {
        &self.bin
    }

    // return git crates source size information
    pub(crate) fn git_crates_source(&self) -> &HashMap<String, u64> {
        &self.git_crates_source
    }

    // return registry crates source size information
    pub(crate) fn registry_crates_source(&self) -> &HashMap<String, u64> {
        &self.registry_crates_source
    }

    // return git crates archive size information
    pub(crate) fn git_crates_archive(&self) -> &HashMap<String, u64> {
        &self.git_crates_archive
    }

    // return registry crates archive size information
    pub(crate) fn registry_crates_archive(&self) -> &HashMap<String, u64> {
        &self.registry_crates_archive
    }

    // add bin information to CrateDetail
    pub(crate) fn add_bin(&mut self, bin_name: String, size: u64) {
        self.bin.insert(bin_name, size);
    }

    // add git crate source information to CrateDetail
    pub(crate) fn add_git_crate_source(&mut self, crate_name: String, size: u64) {
        if let Some(crate_size) = self.git_crates_source.get_mut(&crate_name) {
            *crate_size += size;
        } else {
            self.git_crates_source.insert(crate_name, size);
        }
    }

    // add registry crate source information to CrateDetail
    pub(crate) fn add_registry_crate_source(&mut self, crate_name: String, size: u64) {
        if let Some(crate_size) = self.registry_crates_source.get_mut(&crate_name) {
            *crate_size += size;
        } else {
            self.registry_crates_source.insert(crate_name, size);
        }
    }

    // add git crate archive information to CrateDetail
    pub(crate) fn add_git_crate_archive(&mut self, crate_name: String, size: u64) {
        if let Some(crate_size) = self.git_crates_archive.get_mut(&crate_name) {
            *crate_size += size;
        } else {
            self.git_crates_archive.insert(crate_name, size);
        }
    }

    // add registry crate archive information to CrateDetail
    pub(crate) fn add_registry_crate_archive(&mut self, crate_name: String, size: u64) {
        if let Some(crate_size) = self.registry_crates_archive.get_mut(&crate_name) {
            *crate_size += size;
        } else {
            self.registry_crates_archive.insert(crate_name, size);
        }
    }

    // find size of certain git crate source in KB
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn find_size_git_source(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.git_crates_source.get(crate_name) {
            (*size as f64) / 1000_f64.powi(2)
        } else {
            0.0
        }
    }

    // find size of certain registry source in KB
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn find_size_registry_source(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.registry_crates_source.get(crate_name) {
            (*size as f64) / 1000_f64.powi(2)
        } else {
            0.0
        }
    }

    // find size of certain git crate archive in KB
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn find_size_git_archive(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.git_crates_archive.get(crate_name) {
            (*size as f64) / 1000_f64.powi(2)
        } else {
            0.0
        }
    }

    // find size of certain registry archive in KB
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn find_size_registry_archive(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.registry_crates_archive.get(crate_name) {
            (*size as f64) / 1000_f64.powi(2)
        } else {
            0.0
        }
    }

    // return certain git crate total size in KB
    pub(crate) fn find_size_git_all(&self, crate_name: &str) -> f64 {
        self.find_size_git_archive(crate_name) + self.find_size_git_source(crate_name)
    }

    // return certain registry crate total size in KB
    pub(crate) fn find_size_registry_all(&self, crate_name: &str) -> f64 {
        self.find_size_registry_archive(crate_name) + self.find_size_registry_source(crate_name)
    }

    // find crate size if location/title is given in KB
    pub(crate) fn find(&self, crate_name: &str, location: &str) -> f64 {
        if location.contains("REGISTRY") {
            self.find_size_registry_all(crate_name)
        } else if location.contains("GIT") {
            self.find_size_git_all(crate_name)
        } else {
            0.0
        }
    }
}
