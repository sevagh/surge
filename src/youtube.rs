use std::io::Read;

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
            youtube_api_key,
            max_results,
        }
    }

    pub fn find_related_tracks(&self, video_id: &str) -> String {
        self.hyper_request(format!("{0}/search?part=snippet&relatedToVideoId={1}&type=video",
                                   YT_API_URL,
                                   video_id))
    }

    pub fn search_video(&self, keywords: String) -> String {
        let results = self.hyper_request(format!("{0}/search?part=snippet&q={1}&type=video",
                                                 YT_API_URL,
                                                 keywords.replace(" ", "+")));
        match serde_json::from_str::<serde_json::Value>(&results) {
            Ok(x) => println!("{:?}", x),
            Err(e) => panic!(e),
        }
        results
    }

    pub fn gen_download_url(&self, video_id: &str) -> String {
        format!("{0}{1}", YT_VID_BASE_URL, video_id)
    }

    fn hyper_request(&self, url: String) -> String {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);
        let res = client
            .get(format!("{0}&maxResults={1}&key={2}",
                         url,
                         self.max_results,
                         self.youtube_api_key)
                         .as_str())
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
}
