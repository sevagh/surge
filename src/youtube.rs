use std::io::{self, Read, copy};
use std::fs::File;
use std::path::PathBuf;

use backend::{BackendSearchResult, Backend};

use hyper::{self, Client};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use serde_json;
use tempdir::TempDir;

const YT_API_URL: &'static str = "https://www.googleapis.com/youtube/v3";
const YT_VID_BASE_URL: &'static str = "https://www.youtube.com/watch?v=";

pub struct YoutubePlayer {
    tmp_dir: TempDir,
    youtube_api_key: String,
    max_results: u32,
}

impl YoutubePlayer {
    pub fn new(youtube_api_key: String, max_results: u32) -> YoutubePlayer {
        YoutubePlayer {
            youtube_api_key: youtube_api_key,
            max_results: max_results,
            tmp_dir: TempDir::new("surge").expect("Couldn't make tempdir"),
        }
    }
}

impl Backend for YoutubePlayer {
    fn find_related_tracks(&self, video_id: &str) -> Vec<BackendSearchResult> {
        hyper_request(format!("{0}/search?part=snippet&relatedToVideoId={1}&type=video",
                              YT_API_URL,
                              video_id)
                              .as_str(),
                      self.max_results,
                      &self.youtube_api_key);
        Vec::new()
    }

    fn search(&self, keywords: &str) -> Vec<BackendSearchResult> {
        let api_result = hyper_request(format!("{0}/search?part=snippet&q={1}&type=video",
                                               YT_API_URL,
                                               keywords.replace(" ", "+"))
                                               .as_str(),
                                       self.max_results,
                                       &self.youtube_api_key);
        let results: Vec<BackendSearchResult>;
        match serde_json::from_str::<serde_json::Value>(&api_result) {
            Ok(ref x) => {
                results = x["items"]
                    .as_array()
                    .expect("Didn't get expected response from youtube api")
                    .iter()
                    .map(|video_obj| {
                        let title =
                            String::from(video_obj["snippet"]["title"]
                                             .as_str()
                                             .expect("Youtube response didn't contain title"));
                        let id = String::from(video_obj["id"]["videoId"]
                                                  .as_str()
                                                  .expect("Youtube response didn't contain id"));

                        let thumbnail = match video_obj["snippet"]["thumbnails"]["default"]["url"]
                                  .as_str() {
                            Some(x) => {
                                match download_thumbnail(x, &id, &self.tmp_dir) {
                                    Ok(x) => Some(x),
                                    Err(_) => None,
                                }
                            }
                            None => None,
                        };

                        BackendSearchResult {
                            id: id,
                            title: title,
                            thumbnail: thumbnail,
                        }
                    })
                    .collect::<Vec<_>>();
            }
            Err(e) => panic!(e),
        }
        results
    }

    fn gen_download_url(&self, video_id: &str) -> String {
        format!("{0}{1}", YT_VID_BASE_URL, video_id)
    }
}

fn hyper_request(url: &str, max_results: u32, api_key: &str) -> String {
    let ssl = NativeTlsClient::new().expect("Couldn't make TLS client");
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    let res = client
        .get(format!("{0}&maxResults={1}&key={2}", url, max_results, api_key).as_str())
        .send();
    let mut ret = String::new();
    match res {
        Ok(mut res) => {
            match res.read_to_string(&mut ret) {
                Ok(_) => (),
                Err(e) => panic!(e),
            }
        }
        Err(e) => panic!(e),
    }
    ret
}

enum ThumbnailError {
    HyperError(hyper::Error),
    IoError(io::Error),
}

fn download_thumbnail(url: &str, uid: &str, tmp: &TempDir) -> Result<PathBuf, ThumbnailError> {
    let file_path = tmp.path()
        .join(format!("{0}_{1}",
                      uid,
                      url.rsplitn(2, '/').collect::<Vec<&str>>()[0]));
    let file_path_copy = file_path.clone();
    println!("FILE PATH: {:?}", file_path);
    let mut tmp_file = File::create(file_path).expect("Couldn't make temp file");

    let ssl = NativeTlsClient::new().expect("Couldn't make TLS client");
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    let res = client.get(url).send();
    match res {
        Ok(mut res) => {
            match copy(&mut res, &mut tmp_file) {
                Ok(_) => (),
                Err(e) => return Err(ThumbnailError::IoError(e)),
            }
        }
        Err(e) => return Err(ThumbnailError::HyperError(e)),
    }
    Ok(file_path_copy)
}
