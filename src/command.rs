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
}

impl<'a> CommandCenter<'a> {
    pub fn for_youtube(youtube_api_key: String,
                       max_results: u32,
                       download_path: String,
                       out: StdoutLock<'a>)
                       -> CommandCenter<'a> {
        CommandCenter {
            backend: Box::new(YoutubePlayer::new(youtube_api_key, max_results)),
            currents: vec![],
            current: None,
            ytdl: YoutubeDl::new(download_path),
            sink: default_sink(),
            out: out,
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
                self.find_related();
                self.select_interactive();
            }
            "search" => {
                if cmd_split.len() == 2 {
                    self.search(cmd_split[1]);
                    self.select_interactive();
                } else {
                    println!("Please enter non-empty search terms");
                }
            }
            _ => println!("Unrecognized command!"),
        }
    }

    fn select_interactive(&mut self) {
        for (i, x) in self.currents.iter().enumerate() {
            writeln!(self.out, "{0}: {1}", i, x.title).expect("Couldn't write to stdout");
            display_png(x.thumbnail.as_ref(), &mut self.out);
            writeln!(self.out, "").expect("Coudln't write to stdout");
        }
        let sel: usize = read!();
        self.current = self.currents.get(sel).cloned()
    }

    fn search(&mut self, search: &str) {
        self.currents = self.backend.search(search)
    }

    fn find_related(&mut self) {
        match self.current {
            Some(ref x) => self.currents = self.backend.find_related_tracks(x.id.as_str()),
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
    }
}
