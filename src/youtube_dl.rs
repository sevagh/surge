use std::process::Command;

const YT_VID_BASE_URL: &'static str = "https://www.youtube.com/watch?v=";

pub fn download_audio_from_youtube(video_id: &str, dl_path: &str) {
    let dl_opt = format!("{0}/%(title)s.%(ext)s", dl_path);
    let dl_url = format!("{0}{1}", YT_VID_BASE_URL, video_id);

    let output = Command::new("youtube-dl")
        .args(&["--extract-audio",
                "--audio-format", "best",
                "--audio-quality", "0",
                "-o", &dl_opt,
                &dl_url])
        .output()
        .expect("Failed to run youtube-dl command");

    println!("{:?}", output);
}
