#![crate_type = "bin"]
#![feature(const_fn)]
#![feature(drop_types_in_const)]

#[macro_use]
extern crate lazy_static;
extern crate serde_json;
extern crate hyper;
extern crate hyper_native_tls;
extern crate rustyline;
extern crate regex;
extern crate termimage;
extern crate image;
extern crate term_size;
extern crate mpv;
extern crate app_dirs;
extern crate app_setup;
extern crate ini;

mod youtube;
mod download;
mod command;
mod backend;
mod player;

use command::CommandCenter;
use backend::MasterBackend;
use download::Downloader;
use player::AudioPlayer;

use app_dirs::*;
use app_setup::appsetup;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use ini::Ini;

use std::io::stdout;
use std::collections::HashMap;

const SURGE_APP_INFO: AppInfo = AppInfo {
    name: "surge",
    author: "Sevag Hanssian",
};
const SURGE_PROMPT: &'static str = "surge ♫ ";
const SURGE_CONF: &'static str = "surge.ini";

fn main() {
    let out = stdout();

    let mut history_path = get_app_root(AppDataType::UserCache, &SURGE_APP_INFO)
        .expect("Couldn't get user cache dir");
    history_path.push("history.txt");

    let desired_config: HashMap<Option<&str>, Vec<&str>> = [(Some("global"), vec!["yt_api_key"])]
        .iter()
        .cloned()
        .collect();

    let mut conf_file_path = get_app_root(AppDataType::UserConfig, &SURGE_APP_INFO)
        .expect("Couldn't get user config dir");
    conf_file_path.push(SURGE_CONF);

    let config = if conf_file_path.exists() {
        Ini::load_from_file(conf_file_path).expect("Couldn't load ini from config file")
    } else {
        appsetup(&SURGE_APP_INFO, desired_config, SURGE_CONF)
    };

    let yt_api_key = config
        .section(Some("global"))
        .expect("Couldn't get None section of ini file")
        .get("yt_api_key")
        .expect("Missing yt_api_key config");

    let mut backend = MasterBackend::new(yt_api_key);
    let mut player = AudioPlayer::new();
    let mut dloader = Downloader::new(
        app_dir(AppDataType::UserData, &SURGE_APP_INFO, "music")
            .expect("Couldn't get user data dir"),
        app_dir(AppDataType::UserCache, &SURGE_APP_INFO, "thumbnails")
            .expect("Couldn't get user cache dir"),
    );

    let mut cmd = CommandCenter::new(out.lock(), &mut player, &mut dloader, &mut backend);

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
