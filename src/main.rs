#![crate_type = "bin"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate hyper;
extern crate hyper_native_tls;
extern crate toml;

mod youtube_dl;

use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use youtube_dl::download_audio_from_youtube;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

const YT_API_URL: &'static str = "https://www.googleapis.com/youtube/v3";

#[derive(Deserialize)]
struct Config {
    download_path: String,
    keys: Keys,
}

#[derive(Deserialize)]
struct Keys {
    youtube: String,
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

    let video_id = "mQuIy3KWpRw&";
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    let res = client.get(format!("{0}/search?part=snippet&relatedToVideoId={1}&type=video&key={2}", YT_API_URL, video_id, config.keys.youtube).as_str())
            .send();
    match res {
        Ok(res) => println!("Response: {}", res.status),
        Err(e) => println!("Err: {:?}", e)
    }

    download_audio_from_youtube(video_id, config.download_path.as_str());
}
