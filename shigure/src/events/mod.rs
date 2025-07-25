use rainmeter::RainmeterContext;

use crate::OverlayMeter;

impl OverlayMeter {
    pub fn poll_updates(&mut self, rm: &RainmeterContext) {
        // Tick the music player service to update its state
        crate::services::music_player::tick_music_player();
    }
}
