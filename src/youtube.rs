use std::io::Read;

use backend::{BackendSearchResult, Backend};
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

use serde_json;

const YT_API_URL: &'static str = "https://www.googleapis.com/youtube/v3";
const YT_VID_BASE_URL: &'static str = "https://www.youtube.com/watch?v=";

pub struct YoutubePlayer {
    youtube_api_key: String,
    max_results: u32,
}

impl YoutubePlayer {
    pub fn new(youtube_api_key: String, max_results: u32) -> YoutubePlayer {
        YoutubePlayer {
            youtube_api_key: youtube_api_key,
            max_results: max_results,
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
                    .unwrap()
                    .iter()
                    .map(|video_obj| {
                        BackendSearchResult {
                            id: String::from(video_obj["id"]["videoId"].as_str().unwrap()),
                            title: String::from(video_obj["snippet"]["title"].as_str().unwrap()),
                            thumbnail: video_obj["snippet"]["thumbnails"]["default"]["url"]
                                .as_str()
                                .map(str::to_string),
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
    let ssl = NativeTlsClient::new().unwrap();
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
