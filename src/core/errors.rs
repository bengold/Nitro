use thiserror::Error;

#[derive(Error, Debug)]
pub enum NitroError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Formula parse error: {0}")]
    FormulaParse(String),

    #[error("Dependency resolution failed: {0}")]
    DependencyResolution(String),

    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Tap error: {0}")]
    TapError(String),

    #[error("Search error: {0}")]
    SearchError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] sled::Error),

    #[error("Tantivy error: {0}")]
    Tantivy(#[from] tantivy::TantivyError),

    #[error("General error: {0}")]
    General(#[from] anyhow::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type NitroResult<T> = Result<T, NitroError>;