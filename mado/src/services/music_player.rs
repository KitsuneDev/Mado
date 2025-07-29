use schemars::JsonSchema;
use serde::Serialize;

pub trait MusicPlayerService {
    fn play(&self);
    fn pause(&self);
    fn next(&self);
    fn previous(&self);
    /// Sets the volume to a percentage (0.0 to 1.0).
    fn set_volume(&self, volume: f64);
    /// Seeks to a position in the track, where position is a percentage (0.0 to 1.0).
    fn seek_absolute(&self, position: f64);
    // Forces the service to update its state.
    // A status update event will also be raised.
    fn get_data(&self) -> MusicPlayerState;
}

#[derive(Debug, Clone, Serialize, PartialEq, JsonSchema)]
pub enum MusicPlayerStatus {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MusicPlayerState {
    pub is_connected: bool,
    pub player: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// URL to the album cover image
    pub cover: String,
    /// Duration in seconds
    pub duration: String,
    /// Position in seconds
    pub position: String,
    /// Prrogress Percentage (0.0 to 1.0)
    pub progress: f64,
    /// Volume Percentage (0.0 to 1.0)
    pub volume: f64,
    pub status: MusicPlayerStatus,
}

impl PartialEq for MusicPlayerState {
    fn eq(&self, other: &Self) -> bool {
        self.is_connected == other.is_connected &&
        //self.player == other.player && -- This seems to glitch with a few players
        self.title == other.title &&
        self.artist == other.artist &&
        self.album == other.album &&
        self.cover == other.cover &&
        self.duration == other.duration &&
        self.position == other.position &&
        // Compare f64 fields with epsilon for floating-point precision
        (self.progress - other.progress).abs() < f64::EPSILON &&
        (self.volume - other.volume).abs() < f64::EPSILON &&
        self.status == other.status
    }
}
