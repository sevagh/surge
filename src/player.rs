use mpv::{MpvHandler, MpvHandlerBuilder, Event};

use std::thread;
use std::time::Duration;

pub struct AudioPlayer {
    mpv: MpvHandler,
}

impl AudioPlayer {
    pub fn new() -> AudioPlayer {
        AudioPlayer {
            mpv: MpvHandlerBuilder::new()
                .expect("Couldn't initialize MpvHandlerBuilder")
                .build()
                .expect("Couldn't build MpvHandler"),
        }
    }

    pub fn queue(&mut self, new: String) {
        self.mpv
            .command(&["loadfile", &new, "append-play"])
            .expect("Error loading file");
    }

    pub fn queue_and_play(&mut self, new: String) {
        self.mpv
            .command(&["loadfile", &new, "replace"])
            .expect("Error loading file");
    }

    pub fn stop(&mut self) {
        self.mpv.command(&["stop"]).expect("Error stopping mpv");
    }

    pub fn pause(&mut self) {
        self.mpv
            .set_property("pause", true)
            .expect("Toggling pause property");
    }

    pub fn resume(&mut self) {
        self.mpv
            .set_property("pause", false)
            .expect("Toggling pause property");
    }

    pub fn loop_(&mut self) {
        let next_loop = match self.mpv.get_property::<&str>("loop-file") {
            Ok(x) => {
                if x == "inf" || x == "yes" {
                    "no"
                } else if x == "no" || x == "1" {
                    "inf"
                } else {
                    panic!("Unexpected value for loop-file property")
                }
            }
            Err(e) => panic!(e),
        };
        self.mpv
            .set_property("loop-file", next_loop)
            .expect("Toggling loop-file property");
    }

    fn get_time_remain(&mut self) -> Option<u64> {
        match self.mpv.get_property::<i64>("time-remain") {
            Ok(x) => Some(x as u64),
            Err(_) => None,
        }
    }

    pub fn queue_on_file_event_blocking(&mut self, new: String) {
        loop {
            if let Some(Event::FileLoaded) = self.mpv.wait_event(0.0) {
                let sleep_time = self.get_time_remain().unwrap_or(90);
                println!("Queued next song... sleeping {:?}", sleep_time);
                self.queue(new);
                thread::sleep(Duration::from_secs(sleep_time));
                break;
            }
        }
    }
}
