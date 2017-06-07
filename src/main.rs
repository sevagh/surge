#![crate_type = "bin"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate hyper_native_tls;
extern crate toml;
extern crate rustyline;

mod youtube;
mod youtube_dl;

use youtube_dl::YoutubeDl;
use youtube::YoutubePlayer;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

#[derive(Deserialize)]
struct Config {
    download_path: String,
    youtube: Yt,
}

#[derive(Deserialize)]
struct Yt {
    api_key: String,
    max_results: u32,
}

fn main() {
    let file_exists = |ref path| if std::fs::metadata(path).is_ok() {
        Ok(())
    } else {
        Err(format!("File {0} doesn't exist", path))
    };

    include_str!("../Cargo.toml");
    let default_conf_path = format!("{}/.surge/surgeconf.toml", env!("HOME"));
    let matches = clap_app!(
        @app (app_from_crate!())
            (@arg config: -c --config [conf] default_value(
                default_conf_path.as_str())
        {file_exists} "Sets a custom config file")
    )
            .get_matches();

    let mut config_contents = String::new();
    match File::open(matches.value_of("config").unwrap()) {
        Ok(x) => {
            let mut buf_reader = BufReader::new(x);
            match buf_reader.read_to_string(&mut config_contents) {
                Ok(_) => (),
                Err(e) => panic!(e),
            }
        }
        Err(e) => panic!(e),
    }

    let config: Config = toml::from_str(config_contents.as_str()).unwrap();

    let ytplayer = YoutubePlayer::new(config.youtube.api_key,
                                      config.youtube.max_results);
    let ytdl = YoutubeDl::new(config.download_path);

    let history_path = format!("{}/.surge/history.txt", env!("HOME"));
    let history_path = history_path.as_str();

    let mut rl = Editor::<()>::new();
    if let Err(_) = rl.load_history(history_path) {
        ()
    }
    loop {
        let readline = rl.readline("surge ♫ ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                println!("Line: {}", line);
                //ytplayer.find_related_tracks(&line);
                //ytplayer.download_audio_from(&line);
                ytplayer.search_video(line);
                continue;
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) |
            Err(_) => break,
        }
    }
    rl.save_history(history_path).unwrap();
}
