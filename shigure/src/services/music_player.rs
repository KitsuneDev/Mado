use mado::{
    events::Event,
    services::music_player::{MusicPlayerService, MusicPlayerState, MusicPlayerStatus},
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use wry_cmd::commands;

use crate::{get_rainmeter, raise_event};
struct MusicPlayer;

static INSTANCE: MusicPlayer = MusicPlayer;
static LAST_SONG_INFO: Lazy<Mutex<Option<MusicPlayerState>>> = Lazy::new(|| Mutex::new(None));

#[commands]
impl MusicPlayerService for MusicPlayer {
    fn play(&self) {
        if let Some(rm) = get_rainmeter() {
            rm.log(rainmeter::RmLogLevel::LogNotice, "Playing music");
            rm.execute("[!CommandMeasure \"MadoWNPTitle\" \"Play\"]");
        }
    }

    fn pause(&self) {
        if let Some(rm) = get_rainmeter() {
            rm.log(rainmeter::RmLogLevel::LogNotice, "Pausing music");
            rm.execute("[!CommandMeasure \"MadoWNPTitle\" \"Pause\"]");
        }
    }

    fn next(&self) {
        if let Some(rm) = get_rainmeter() {
            rm.log(rainmeter::RmLogLevel::LogNotice, "Next song");
            rm.execute("[!CommandMeasure \"MadoWNPTitle\" \"Next\"]");
        }
    }

    fn previous(&self) {
        if let Some(rm) = get_rainmeter() {
            rm.log(rainmeter::RmLogLevel::LogNotice, "Next song");
            rm.execute("[!CommandMeasure \"MadoWNPTitle\" \"Previous\"]");
        }
    }

    fn set_volume(&self, volume: f64) {
        if let Some(rm) = get_rainmeter() {
            rm.log(rainmeter::RmLogLevel::LogNotice, "Set Volume");
            rm.execute(&format!(
                "[!CommandMeasure \"MadoWNPTitle\" \"SetVolume {volume}\"]"
            ));
        }
    }

    fn seek_absolute(&self, position: f64) {
        if let Some(rm) = get_rainmeter() {
            rm.log(rainmeter::RmLogLevel::LogNotice, "Set Position");
            rm.execute(&format!(
                "[!CommandMeasure \"MadoWNPTitle\" \"SetPosition {position}\"]"
            ));
        }
    }

    fn get_data(&self) -> MusicPlayerState {
        tick_music_player();
        return LAST_SONG_INFO
            .lock()
            .as_ref()
            .cloned()
            .unwrap_or_else(get_current_song);
    }
}

pub fn tick_music_player() {
    let current_song = get_current_song();
    let mut last_song_info = LAST_SONG_INFO.lock();

    // If the song info has changed, update it and raise an event
    if *last_song_info != Some(current_song.clone()) {
        *last_song_info = Some(current_song.clone());
        if let Some(rm) = get_rainmeter() {
            raise_event(Event {
                kind: "music_player_update",
                value: current_song.clone(),
            });
        }
    }
}

fn get_current_song() -> MusicPlayerState {
    if let Some(rm) = get_rainmeter() {
        //rm.log(rainmeter::RmLogLevel::LogNotice, "Getting current song");
        //rm.execute("[!CommandMeasure \"MadoWNPTitle\" \"GetCurrentSong\"]");
        return MusicPlayerState {
            is_connected: rm.read_int("MadoWNPStatus", 0) == 1,
            player: rm.read_string("MadoWNPPlayer", "No Player"),
            title: rm.read_string("MadoWNPTitle", ""),
            artist: rm.read_string("MadoWNPArtist", ""),
            album: rm.read_string("MadoWNPAlbum", ""),
            cover: rm.read_string("MadoWNPAlbumCover", ""),
            duration: rm.read_string("MadoWNPDuration", "00:00"),
            position: rm.read_string("MadoWNPPosition", "00:00"),
            progress: rm.read_double("MadoWNPProgress", 0f64),

            volume: rm.read_double("MadoWNPVolume", 0f64),
            status: match rm.read_int("MadoWNPState", 0) {
                1 => MusicPlayerStatus::Playing,
                2 => MusicPlayerStatus::Paused,
                _ => MusicPlayerStatus::Stopped,
            },
        };
    } else {
        return MusicPlayerState {
            is_connected: false,
            player: "No Rainmeter".to_string(),
            title: "".to_string(),
            artist: "".to_string(),
            album: "".to_string(),
            cover: "".to_string(),
            duration: "00:00".to_string(),
            position: "00:00".to_string(),
            progress: 0.0,
            volume: 0.0,
            status: MusicPlayerStatus::Stopped,
        };
    }
}
