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
extern crate regex;
extern crate termimage;
extern crate image;
extern crate term_size;
extern crate mpv;

mod youtube;
mod download;
mod command;
mod backend;
mod player;

use backend::{MasterBackend, BackendType};
use command::CommandCenter;
use download::Downloader;

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
    download_path: Option<String>,
    youtube_api_key: Option<String>,
}

fn main() {
    let conf_dir = format!("{}/{}", env!("HOME"), SURGE_CONF_DIR);
    let cache_dir = format!("{}/{}", env!("HOME"), SURGE_CACHE_DIR);
    let thumbnail_cache_dir = format!("{}/thumbnails", cache_dir);
    let music_dl_dir = format!("{}/music", cache_dir);
    let conf_path = format!("{}/surgeconf.toml", conf_dir);
    let history_path = format!("{}/history.txt", cache_dir);

    create_dir_all(conf_dir).expect("Couldn't create conf dir");
    create_dir_all(thumbnail_cache_dir.clone()).expect("Coudln't create cache dir");
    create_dir_all(music_dl_dir.clone()).expect("Coudln't create music dir");

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

    let mut backend = MasterBackend::new(config.youtube_api_key, &client);
    backend.set_type(BackendType::Youtube);

    let dlpath = match config.download_path {
        Some(x) => x,
        None => music_dl_dir,
    };

    let downloader = Downloader::new(dlpath, thumbnail_cache_dir, &client);

    let mut cmd = CommandCenter::new(&backend, downloader, out.lock());

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
            Err(_) => {
                cmd.stop();
                break;
            }
        }
    }
    rl.save_history(&history_path).unwrap();
}
