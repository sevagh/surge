#![crate_type = "lib"]

extern crate rustyline;
extern crate app_dirs;
extern crate ini;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use ini::Ini;
use app_dirs::*;

use std::collections::HashMap;

pub fn appsetup(
    app_info: &AppInfo,
    config: HashMap<Option<&str>, Vec<&str>>,
    config_file_name: &str,
) -> Ini {
    let mut app_conf = app_root(AppDataType::UserConfig, app_info).expect(
        "Couldn't create platform-specific config dir",
    );
    app_conf.push(config_file_name);

    let mut conf = Ini::new();

    let prompt = format!("<{0} setup>", app_info.name);

    let mut rl = Editor::<()>::new();
    for (section, keys) in config {
        for k in keys {
            'outer: loop {
                let readline = rl.readline(&format!("{0} Enter value for {1}: ", prompt, k));
                match readline {
                    Ok(line) => {
                        if line != "" {
                            conf.with_section(section).set(k, line);
                            break 'outer;
                        }
                        continue;
                    }
                    Err(ReadlineError::Interrupted) => continue,
                    Err(ReadlineError::Eof) |
                    Err(_) => break,
                }
            }
        }
    }

    println!("First time setup complete!\n");
    conf.write_to_file(app_conf.as_path()).expect(
        "Couldn't write to conf file",
    );
    conf
}
