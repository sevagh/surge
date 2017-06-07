use youtube::YoutubePlayer;
use youtube_dl::YoutubeDl;
use backend::*;

use std::boxed::Box;

pub struct CommandCenter {
    ytdl: YoutubeDl,
    backend: Box<Backend>,
    currents: Vec<BackendSearchResult>,
    current: Option<BackendSearchResult>,
}

impl CommandCenter {
    pub fn for_youtube(youtube_api_key: String,
                       max_results: u32,
                       download_path: String)
                       -> CommandCenter {
        CommandCenter {
            backend: Box::new(YoutubePlayer::new(youtube_api_key, max_results)),
            currents: vec![],
            current: None,
            ytdl: YoutubeDl::new(download_path),
        }
    }

    pub fn search(&mut self, search: String) {
        self.currents = self.backend.search(search.as_str())
    }

    pub fn select(&mut self, sel: usize) {
        self.current = self.currents.get(sel).cloned()
    }

    pub fn find_related(&mut self) {
        match self.current {
            Some(ref x) => self.currents = self.backend.find_related_tracks(x.id.as_str()),
            None => panic!("No current selection"),
        }
    }

    fn download_audio(&mut self) {
        match self.current {
            Some(ref x) => {
                self.ytdl
                    .download_audio_from_url(self.backend.gen_download_url(x.id.as_str()))
            }
            None => panic!("No current selection"),
        }
    }
}
