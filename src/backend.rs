#[derive(Debug, Clone)]
pub struct BackendSearchResult {
    pub thumbnail: Option<String>,
    pub id: String,
    pub title: String,
}

pub trait Backend {
    fn find_related_tracks(&self, &str) -> Vec<BackendSearchResult>;
    fn search(&self, &str) -> Vec<BackendSearchResult>;
    fn gen_download_url(&self, &str) -> String;
}
