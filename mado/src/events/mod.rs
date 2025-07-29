use serde::Serialize;

use crate::services::music_player::MusicPlayerState;

#[derive(Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum Event {
    MusicUpdate(MusicPlayerState),
    ERROR(ErrorData),
    // Add more variants here
}
#[derive(Serialize)]
pub struct ErrorData {
    pub message: String,
    pub code: u32,
}

pub trait EventRaiser {
    fn raise_event(&self, event: Event);
}
