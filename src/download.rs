use regex::Regex;

use std::process::Command;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Error, copy};

use hyper::Client;

pub struct Downloader<'a> {
    music_dl_path: String,
    thumbnail_cache_path: String,
    client: &'a Client,
}

impl<'a> Downloader<'a> {
    pub fn new(dl_path: String,
               thumbnail_cache_path: String,
               client: &'a Client)
               -> Downloader<'a> {
        Downloader {
            music_dl_path: dl_path,
            thumbnail_cache_path: thumbnail_cache_path,
            client: client,
        }
    }

    pub fn download_audio_from_url(&self, url: String) -> Result<String, Error> {
        let dl_opt = format!("{0}/%(title)s.%(ext)s", self.music_dl_path);

        match Command::new("youtube-dl")
                  .args(&["--extract-audio",
                          "--audio-format",
                          "flac",
                          "--audio-quality",
                          "0",
                          "-o",
                          &dl_opt,
                          &url])
                  .output() {
            Ok(output) => {
                Ok(get_dl_path_from_ytdl_stdout(String::from_utf8_lossy(&output.stdout)
                                                    .into_owned()
                                                    .as_str()))
            }
            Err(e) => Err(e),
        }
    }

    pub fn download_thumbnail(&self, url: Option<&str>, uid: &str) -> Option<PathBuf> {
        let url = match url {
            Some(x) => x,
            None => return None,
        };
        let thumbnail_cache_path_loc = self.thumbnail_cache_path.clone();
        let file_path = PathBuf::from(thumbnail_cache_path_loc).join(format!("{0}_{1}",
                        uid,
                        url.rsplitn(2, '/').collect::<Vec<&str>>()[0]));
        let file_path_copy = file_path.clone();
        if file_path.exists() {
            return Some(file_path_copy);
        }
        let mut tmp_file = File::create(file_path).expect("Couldn't make temp file");

        let res = self.client.get(url).send();
        match res {
            Ok(mut res) => {
                match copy(&mut res, &mut tmp_file) {
                    Ok(_) => (),
                    Err(_) => return None,
                }
            }
            Err(_) => return None,
        }
        Some(file_path_copy)
    }
}

fn get_dl_path_from_ytdl_stdout(out: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("Destination:.*flac\n").unwrap();
    }
    let cap = RE.captures(out).unwrap();
    let mut ret = String::from(&cap[0][13..]);
    ret.pop();
    ret
}
