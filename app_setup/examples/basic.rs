extern crate app_dirs;
extern crate app_setup;

use app_dirs::*;
use app_setup::appsetup;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};

fn main() {
    let app_info = AppInfo {
        name: "MyExample",
        author: "Sevag Hanssian",
    };

    let desired_config: HashMap<Option<&str>, Vec<&str>> =
        [
            (None, vec!["top_level_key_1"]),
            (Some("section_2"), vec!["s2_key_1", "s2_key_2"]),
        ].iter()
            .cloned()
            .collect();

    appsetup(&app_info, desired_config, "myexample.ini");
    let mut conf_file =
        get_app_root(AppDataType::UserConfig, &app_info).expect("Couldn't get conf dir");
    conf_file.push("myexample.ini");

    let f = File::open(conf_file.clone()).expect("Couldn't open conf.ini file");

    println!("Successfully opened: {:?}.\nContents:", conf_file);

    for l in BufReader::new(f).lines() {
        println!("{:?}", l);
    }
}
