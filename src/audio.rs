use rodio;
use std::fs::File;
use std::io::BufReader;

pub struct AudioPlayer {
    sink: Option<rodio::Sink>,
}

impl AudioPlayer {
    pub fn new() -> AudioPlayer {
        AudioPlayer { sink: default_sink() }
    }

    pub fn pause(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.pause();
        }
    }

    pub fn queue(&mut self, file: File) {
        if let Some(ref sink) = self.sink {
            sink
                .append(rodio::Decoder::new(BufReader::new(file))
                        .expect("Coudln't make rodio decoder"));
        }
    }

    pub fn play(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.play();
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.pause();
            sink.detach();
            self.sink = default_sink();
        }
    }
}

fn default_sink() -> Option<rodio::Sink> {
    Some(rodio::Sink::new(&rodio::get_default_endpoint().expect("Couldn't make rodio endpoint")))
}
