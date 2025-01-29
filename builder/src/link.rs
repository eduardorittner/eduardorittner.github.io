use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Link {
    pub link: String,
    pub file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RelativeLink(pub Link);

#[derive(Debug, Clone)]
pub struct UrlLink(pub Link);
