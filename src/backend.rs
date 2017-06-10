use hyper::Client;

use youtube::YoutubeBackend;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BackendType {
    Youtube,
    Uninitialized,
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
    fn gen_download_url(&self, &str) -> String;
}

pub struct MasterBackend<'a> {
    btype: BackendType,
    ytb: Option<YoutubeBackend<'a>>,
}

impl<'a> MasterBackend<'a> {
    pub fn new(ytapi: Option<String>, client: &'a Client) -> MasterBackend<'a> {
        MasterBackend {
            btype: BackendType::Uninitialized,
            ytb: match ytapi {
                Some(x) => Some(YoutubeBackend::new(x, client)),
                None => None,
            },
        }
    }

    pub fn set_type(&mut self, btype: BackendType) {
        self.btype = btype;
    }
}

impl<'a> Backend for MasterBackend<'a> {
    fn find_related_tracks(&self, x: &str) -> Vec<BackendSearchResult> {
        match self.btype {
            BackendType::Youtube => {
                match self.ytb {
                    Some(ref ytb) => ytb.find_related_tracks(x),
                    None => panic!("No youtube backend"),
                }
            }
            _ => panic!("Backend type not initialized"),
        }
    }

    fn search(&self, x: &str) -> Vec<BackendSearchResult> {
        match self.btype {
            BackendType::Youtube => {
                match self.ytb {
                    Some(ref ytb) => ytb.search(x),
                    None => panic!("No youtube backend"),
                }
            }
            _ => panic!("Backend type not initialized"),
        }
    }

    fn gen_download_url(&self, x: &str) -> String {
        match self.btype {
            BackendType::Youtube => {
                match self.ytb {
                    Some(ref ytb) => ytb.gen_download_url(x),
                    None => panic!("No youtube backend"),
                }
            }
            _ => panic!("Backend type not initialized"),
        }
    }
}
