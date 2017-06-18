use youtube::YoutubeBackend;

use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BackendType {
    Youtube,
}

#[derive(Debug, Clone)]
pub struct BackendSearchResult {
    pub thumbnail: Option<String>,
    pub id: String,
    pub title: String,
}

pub trait Backend {
    fn find_related_tracks(&self, &str) -> Vec<BackendSearchResult>;
    fn search(&self, &str) -> Vec<BackendSearchResult>;
}

pub struct MasterBackend {
    btype: BackendType,
    ytb: YoutubeBackend,
}

impl MasterBackend {
    pub fn new(yt_api_key: &str) -> MasterBackend {
        let ssl = NativeTlsClient::new().expect("Couldn't make TLS client");
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        MasterBackend {
            btype: BackendType::Youtube,
            ytb: YoutubeBackend::new(String::from(yt_api_key), client),
        }
    }
}

impl Backend for MasterBackend {
    fn find_related_tracks(&self, x: &str) -> Vec<BackendSearchResult> {
        match self.btype {
            BackendType::Youtube => self.ytb.find_related_tracks(x),
        }
    }

    fn search(&self, x: &str) -> Vec<BackendSearchResult> {
        match self.btype {
            BackendType::Youtube => self.ytb.search(x),
        }
    }
}
