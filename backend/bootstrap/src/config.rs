use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapConfig {
    pub database_path: PathBuf,
}

impl BootstrapConfig {
    pub fn new(database_path: impl Into<PathBuf>) -> Self {
        Self {
            database_path: database_path.into(),
        }
    }

    pub fn from_env() -> Self {
        std::env::var_os("LH_DATABASE_PATH")
            .map(PathBuf::from)
            .map(Self::new)
            .unwrap_or_default()
    }
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self::new("data/users.db")
    }
}
