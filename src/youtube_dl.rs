use regex::Regex;

use std::process::Command;
use std::io::Error;

pub struct YoutubeDl {
    dl_path: String,
}

impl YoutubeDl {
    pub fn new(dl_path: String) -> YoutubeDl {
        YoutubeDl { dl_path }
    }

    pub fn download_audio_from_url(&self, url: String) -> Result<String, Error> {
        let dl_opt = format!("{0}/%(title)s.%(ext)s", self.dl_path);

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
                    Ok(get_dl_path_from_ytdl_stdout(String::from_utf8_lossy(&output.stdout).into_owned().as_str()))
                }
                Err(e) => Err(e),
        }
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
