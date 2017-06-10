use std::io::Read;

use backend::{BackendSearchResult, Backend};

use hyper::Client;
use serde_json;

const YT_API_URL: &'static str = "https://www.googleapis.com/youtube/v3";
const YT_VID_BASE_URL: &'static str = "https://www.youtube.com/watch?v=";

pub struct YoutubePlayer<'a> {
    youtube_api_key: String,
    client: &'a Client,
}

impl<'a> YoutubePlayer<'a> {
    pub fn new(youtube_api_key: String, client: &'a Client) -> YoutubePlayer<'a> {
        YoutubePlayer {
            youtube_api_key: youtube_api_key,
            client: client,
        }
    }

    fn hyper_request(&self, url: &str) -> String {
        let res = self.client
            .get(format!("{0}&key={1}", url, self.youtube_api_key).as_str())
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

impl<'a> Backend for YoutubePlayer<'a> {
    fn find_related_tracks(&self, video_id: &str) -> Vec<BackendSearchResult> {
        let api_result =
            self.hyper_request(format!("{0}/search?part=snippet&relatedToVideoId={1}&type=video",
                                       YT_API_URL,
                                       video_id)
                                       .as_str());

        yt_json_parser(&api_result)
    }

    fn search(&self, keywords: &str) -> Vec<BackendSearchResult> {
        let api_result = self.hyper_request(format!("{0}/search?part=snippet&q={1}&type=video",
                                                    YT_API_URL,
                                                    keywords.replace(" ", "+"))
                                                    .as_str());
        yt_json_parser(&api_result)
    }


    fn gen_download_url(&self, video_id: &str) -> String {
        format!("{0}{1}", YT_VID_BASE_URL, video_id)
    }
}


fn yt_json_parser(yt_json: &str) -> Vec<BackendSearchResult> {
    let results: Vec<BackendSearchResult>;
    match serde_json::from_str::<serde_json::Value>(yt_json) {
        Ok(ref x) => {
            results = x["items"]
                .as_array()
                .expect("Didn't get expected response from youtube api")
                .iter()
                .map(|video_obj| {
                    let title = String::from(video_obj["snippet"]["title"]
                                                 .as_str()
                                                 .expect("Youtube response didn't contain title"));
                    let id = String::from(video_obj["id"]["videoId"]
                                              .as_str()
                                              .expect("Youtube response didn't contain id"));

                    let thumbnail = video_obj["snippet"]["thumbnails"]["default"]["url"]
                        .as_str()
                        .map(str::to_string);
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
