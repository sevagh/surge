use termimage::ops;
use image::GenericImage;
use term_size::dimensions;

use player::*;
use download::Downloader;
use backend::*;

use std::io::{StdoutLock, Write};
use std::path::PathBuf;

const SCALE_FACTOR: f32 = 0.5;

pub struct CommandCenter<'a> {
    currents: Vec<BackendSearchResult>,
    current: Option<BackendSearchResult>,
    out: StdoutLock<'a>,
    cycle_ctr: usize,
    nodl: bool,
    player: &'a mut AudioPlayer,
    dloader: &'a mut Downloader,
    backend: &'a mut MasterBackend,
}

impl<'a> CommandCenter<'a> {
    pub fn new(
        out: StdoutLock<'a>,
        player: &'a mut AudioPlayer,
        dloader: &'a mut Downloader,
        backend: &'a mut MasterBackend,
    ) -> CommandCenter<'a> {
        CommandCenter {
            currents: vec![],
            current: None,
            out: out,
            cycle_ctr: 0,
            nodl: true,
            player: player,
            dloader: dloader,
            backend: backend,
        }
    }

    pub fn handle_command(&mut self, command: &str) {
        let cmd_split = command.splitn(2, ' ').collect::<Vec<&str>>();
        match cmd_split[0] {
            "" => (),
            "play" => {
                if cmd_split.len() == 2 {
                    self.select(cmd_split[1]);
                    let dl = self.download().expect("Couldn't dl song");
                    self.player.queue_and_play(dl);
                }
                self.player.resume();
            }
            "download" => {
                if self.nodl {
                    println!("Toggling download mode: ON")
                } else {
                    println!("Toggling download mode: OFF")
                }
                self.nodl = !self.nodl;
            }
            "queue" => {
                self.select(cmd_split[1]);
                let dl = self.download().expect("Couldn't dl song");
                self.player.queue(dl);
            }
            "loop" => self.player.loop_(),
            "pause" => self.player.pause(),
            "related" => {
                self.related("");
                self.cycle();
            }
            "cycle" => self.cycle(),
            "now" => {
                self.now();
                self.player.print_time_remain();
            }
            "stop" => self.stop(),
            "search" => {
                if cmd_split.len() == 2 {
                    self.search(cmd_split[1]);
                    self.cycle();
                } else {
                    println!("Please enter non-empty search terms");
                }
            }
            "help" => unimplemented!(),
            _ => println!("Unrecognized command! Try 'help'"),
        }
    }

    fn cycle(&mut self) {
        if self.cycle_ctr > self.currents.len() - 1 {
            self.cycle_ctr = 0;
        }
        match self.currents.get(self.cycle_ctr) {
            Some(x) => {
                println!("{0}: {1}", self.cycle_ctr, x.title);
                display_png(
                    self.dloader.download_thumbnail(
                        x.thumbnail.as_ref().map(String::as_str),
                        &x.id,
                    ),
                    &mut self.out,
                );
            }
            None => panic!("Shouldn't happen"),
        }
        self.cycle_ctr += 1;
    }

    fn now(&mut self) {
        match self.current {
            Some(ref x) => {
                println!("NOW PLAYING: {0}", x.title);
                display_png(
                    self.dloader.download_thumbnail(
                        x.thumbnail.as_ref().map(String::as_str),
                        &x.id,
                    ),
                    &mut self.out,
                );
            }
            None => println!("Nothing currently playing."),
        }
    }

    fn select(&mut self, sel: &str) {
        let sel: usize = sel.parse().expect("Couldn't parse selection as number");
        self.current = Some(self.currents.remove(sel));
        if let Some(ref x) = self.current {
            println!("SELECTED: {0}", x.title);
            display_png(
                self.dloader.download_thumbnail(
                    x.thumbnail.as_ref().map(String::as_str),
                    &x.id,
                ),
                &mut self.out,
            );
        }
    }

    fn search(&mut self, search: &str) {
        self.cycle_ctr = 0;
        self.currents.clear();
        self.currents.append(&mut self.backend.search(search));
    }

    fn related(&mut self, _: &str) {
        match self.current {
            Some(ref x) => {
                self.cycle_ctr = 0;
                self.currents.clear();
                self.currents.append(&mut self.backend.find_related_tracks(
                    x.id.as_str(),
                ));
            }
            None => panic!("No current selection"),
        }
    }

    fn download(&mut self) -> Option<String> {
        match self.current {
            Some(ref mut x) => {
                match self.dloader.download_audio_from_yt(
                    x.id.as_str(),
                    self.nodl,
                ) {
                    Ok(x) => Some(x),
                    Err(_) => None,
                }
            }
            None => None,
        }
    }

    pub fn stop(&mut self) {
        self.player.stop();
    }
}

fn display_png(path: Option<PathBuf>, out: &mut StdoutLock) {
    let path_ = match path {
        Some(x) => x.clone(),
        None => return,
    };
    let tup = &(String::new(), path_);
    let format = ops::guess_format(tup).expect("Couldn't guess format of downloaded thumbnail");
    let img = ops::load_image(tup, format).expect("Couldn't load downloaded thumbnail");

    if let Some((w, h)) = dimensions() {
        let (w, h) = (w as u32, h as u32);
        let img_s = ops::image_resized_size(img.dimensions(), (w, h), true);
        let (w, h) = (
            (SCALE_FACTOR * img_s.0 as f32) as u32,
            (SCALE_FACTOR * img_s.1 as f32) as u32,
        );
        let resized = ops::resize_image(&img, (w, h));
        ops::write_ansi_truecolor(out, &resized);
        writeln!(out, "\x1b[0m").expect("Couldn't write to stdout");
    }
}
