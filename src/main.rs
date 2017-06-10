#![crate_type = "bin"]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate hyper_native_tls;
extern crate toml;
extern crate rustyline;
extern crate rodio;
extern crate regex;
extern crate termimage;
extern crate image;
extern crate term_size;

mod youtube;
mod download;
mod command;
mod backend;
mod audio;

use command::CommandCenter;
use download::Downloader;
use youtube::YoutubePlayer;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

use std::fs::{File, create_dir_all};
use std::io::{stdout, BufReader};
use std::io::prelude::*;

const SURGE_CONF_DIR: &'static str = ".config/surge";
const SURGE_CACHE_DIR: &'static str = ".cache/surge";
const SURGE_PROMPT: &'static str = "surge ♫ ";

#[derive(Deserialize)]
struct Config {
    download_path: String,
    youtube: Yt,
}

#[derive(Deserialize)]
struct Yt {
    api_key: String,
}

fn main() {
    let conf_dir = format!("{}/{}", env!("HOME"), SURGE_CONF_DIR);
    let cache_dir = format!("{}/{}", env!("HOME"), SURGE_CACHE_DIR);
    let thumbnail_cache_dir = format!("{}/thumbnails/", cache_dir);
    let conf_path = format!("{}/surgeconf.toml", conf_dir);
    let history_path = format!("{}/history.txt", cache_dir);

    create_dir_all(conf_dir).expect("Couldn't create conf dir");
    create_dir_all(thumbnail_cache_dir.clone()).expect("Coudln't create cache dir");

    let mut config_contents = String::new();

    match File::open(conf_path) {
        Ok(x) => {
            let mut buf_reader = BufReader::new(x);
            match buf_reader.read_to_string(&mut config_contents) {
                Ok(_) => (),
                Err(e) => panic!(e),
            }
        }
        Err(e) => panic!(e),
    }

    let config: Config = toml::from_str(&config_contents).unwrap();

    let out = stdout();

    let ssl = NativeTlsClient::new().expect("Couldn't make TLS client");
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let yt_backend = YoutubePlayer::new(config.youtube.api_key, &client);
    let downloader = Downloader::new(config.download_path, thumbnail_cache_dir, &client);
    let mut cmd = CommandCenter::new(&yt_backend, downloader, out.lock());

    let mut rl = Editor::<()>::new();
    if rl.load_history(&history_path).is_err() {
        ()
    }
    loop {
        let readline = rl.readline(SURGE_PROMPT);
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                cmd.handle_command(&line);
                continue;
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) |
            Err(_) => break,
        }
    }
    rl.save_history(&history_path).unwrap();
}
