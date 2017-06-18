use regex::Regex;

use std::process::Command;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Error, copy};

use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

pub struct Downloader {
    client: Client,
    music_dir: PathBuf,
    thumbnail_dir: PathBuf,
}

impl Downloader {
    pub fn new(music_dir: PathBuf, thumbnail_dir: PathBuf) -> Downloader {
        let ssl = NativeTlsClient::new().expect("Couldn't make TLS client");
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        Downloader {
            client,
            music_dir,
            thumbnail_dir,
        }
    }

    pub fn download_audio_from_yt(&self, id: &str, nodl: bool) -> Result<String, Error> {
        let dl_opt = format!(
            "{0}/%(title)s.%(ext)s",
            self.music_dir.to_str().expect(
                "Coudln't convert music_dir to str",
            )
        );
        let dl_url = format!("https://www.youtube.com/watch?v={0}", id);

        if nodl {
            return Ok(dl_url);
        }
        match Command::new("youtube-dl")
            .args(
                &[
                    "--extract-audio",
                    "--audio-format",
                    "flac",
                    "--audio-quality",
                    "0",
                    "-o",
                    &dl_opt,
                    &dl_url,
                ],
            )
            .output() {
            Ok(output) => {
                Ok(get_dl_path_from_ytdl_stdout(
                    String::from_utf8_lossy(&output.stdout)
                        .into_owned()
                        .as_str(),
                ))
            }
            Err(e) => Err(e),
        }
    }

    pub fn download_thumbnail(&self, url: Option<&str>, uid: &str) -> Option<PathBuf> {
        let url = match url {
            Some(x) => x,
            None => return None,
        };
        let mut file_path = self.thumbnail_dir.clone();
        file_path.push(format!(
            "{0}_{1}",
            uid,
            url.rsplitn(2, '/').collect::<Vec<&str>>()[0]
        ));
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
        static ref RE: Regex = Regex::new("Destination:.*flac\n").expect("Couldn't recreate regex");
    }
    let cap = RE.captures(out).expect(
        "Didn't find dl path output from ytdl",
    );
    let mut ret = String::from(&cap[0][13..]);
    ret.pop();
    ret
}
