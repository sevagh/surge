use std::process::Command;

pub struct YoutubeDl {
    dl_path: String,
}

impl YoutubeDl {
    pub fn new(dl_path: String) -> YoutubeDl {
        YoutubeDl {
            dl_path
        }
    }

    pub fn download_audio_from_url(&self, url: &str) {
        let dl_opt = format!("{0}/%(title)s.%(ext)s", self.dl_path);

        let output = Command::new("youtube-dl")
            .args(&["--extract-audio",
                    "--audio-format",
                    "best",
                    "--audio-quality",
                    "0",
                    "-o",
                    &dl_opt,
                    url])
            .output()
            .expect("Failed to run youtube-dl command");

        println!("{:?}", output);
    }
}
