use rodio;
use termimage::ops;
use terminal_size::*;
use image::GenericImage;

use youtube::YoutubePlayer;
use youtube_dl::YoutubeDl;
use backend::*;

use std::io::{BufReader, StdoutLock, Write};
use std::boxed::Box;
use std::fs::File;
use std::path::PathBuf;

const SCALE_FACTOR: f32 = 0.5;

pub struct CommandCenter<'a> {
    ytdl: YoutubeDl,
    backend: Box<Backend>,
    currents: Vec<BackendSearchResult>,
    current: Option<BackendSearchResult>,
    sink: rodio::Sink,
    out: StdoutLock<'a>,
    cycle_ctr: usize,
}

impl<'a> CommandCenter<'a> {
    pub fn for_youtube(youtube_api_key: String,
                       download_path: String,
                       out: StdoutLock<'a>)
                       -> CommandCenter<'a> {
        CommandCenter {
            backend: Box::new(YoutubePlayer::new(youtube_api_key)),
            currents: vec![],
            current: None,
            ytdl: YoutubeDl::new(download_path),
            sink: default_sink(),
            out: out,
            cycle_ctr: 0,
        }
    }

    pub fn handle_command(&mut self, command: &str) {
        let cmd_split = command.splitn(2, ' ').collect::<Vec<&str>>();
        match cmd_split[0] {
            "" => (),
            "play" => self.play_current(),
            "pause" => self.pause(),
            "resume" => self.resume(),
            "related" => {
                self.find_related("");
                self.display_next();
            }
            "cycle" => self.display_next(),
            "reset" => self.reset(),
            "now" => self.display_current(),
            "search" => {
                if cmd_split.len() == 2 {
                    self.search(cmd_split[1]);
                    self.display_next();
                } else {
                    println!("Please enter non-empty search terms");
                }
            }
            "select" => self.select(cmd_split[1]),
            "help" => {
                println!("Valid commands:\n\t\
                                search, cycle, select, play, pause, resume, related, help")
            }
            _ => println!("Unrecognized command! Try 'help'"),
        }
    }

    fn display_next(&mut self) {
        if self.cycle_ctr > self.currents.len() - 1 {
            self.cycle_ctr = 0;
        }
        match self.currents.get(self.cycle_ctr) {
            Some(x) => {
                println!("{0}: {1}", self.cycle_ctr, x.title);
                display_png(x.thumbnail.as_ref(), &mut self.out);
            }
            None => panic!("Shouldn't happen"),
        }
        self.cycle_ctr += 1;
    }

    fn reset(&mut self) {
        self.cycle_ctr = 0;
        self.currents.clear();
    }

    fn display_current(&mut self) {
        match self.current {
            Some(ref x) => {
                println!("NOW PLAYING: {0}", x.title);
                display_png(x.thumbnail.as_ref(), &mut self.out);
            }
            None => println!("Nothing currently playing. Use cycle and select"),
        }
    }

    fn select(&mut self, sel: &str) {
        let sel: usize = sel.parse().expect("Couldn't parse selection as number");
        self.current = self.currents.get(sel).cloned()
    }

    fn search(&mut self, search: &str) {
        self.currents.clear();
        self.currents.append(&mut self.backend.search(search));
    }

    fn find_related(&mut self, _: &str) {
        match self.current {
            Some(ref x) => {
                self.currents.clear();
                self.currents
                    .append(&mut self.backend.find_related_tracks(x.id.as_str()));
            }
            None => panic!("No current selection"),
        }
    }

    fn play_current(&mut self) {
        match self.current {
            Some(ref mut x) => {
                let path = download_audio(&self.ytdl, self.backend.gen_download_url(x.id.as_str()));
                match File::open(path) {
                    Ok(file) => {
                        self.sink
                            .append(rodio::Decoder::new(BufReader::new(file))
                                    .expect("Coudln't make rodio decoder"));
                        self.sink.play();
                    }
                    Err(e) => panic!(e),
                }
            }
            None => panic!("No current selection"),
        }
    }

    fn pause(&mut self) {
        self.sink.pause();
    }

    fn resume(&mut self) {
        self.sink.play();
    }
}

fn download_audio(ytdl: &YoutubeDl, url: String) -> String {
    match ytdl.download_audio_from_url(url) {
        Ok(x) => x,
        Err(e) => panic!(e),
    }
}

fn default_sink() -> rodio::Sink {
    rodio::Sink::new(&rodio::get_default_endpoint().expect("Couldn't make rodio endpoint"))
}

fn display_png(path: Option<&PathBuf>, out: &mut StdoutLock) {
    let path_ = match path {
        Some(x) => x.clone(),
        None => return,
    };
    let tup = &(String::new(), path_);
    let format = match ops::guess_format(tup) {
        Ok(x) => x,
        Err(e) => panic!(e),
    };
    let img = match ops::load_image(tup, format) {
        Ok(x) => x,
        Err(e) => panic!(e),
    };

    if let Some((Width(w), Height(h))) = terminal_size() {
        let (w_, h_) = (w as u32, h as u32);
        let img_s = ops::image_resized_size(img.dimensions(), (w_, h_), true);
        let (w__, h__) = ((SCALE_FACTOR * img_s.0 as f32) as u32,
                          (SCALE_FACTOR * img_s.1 as f32) as u32);
        let resized = ops::resize_image(&img, (w__, h__));
        ops::write_ansi_truecolor(out, &resized);
        writeln!(out, "\x1b[0m").expect("Couldn't write to stdout");
    }
}
