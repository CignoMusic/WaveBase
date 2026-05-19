use crate::error::AppError;

pub enum PlaybackCommand {
    Play(String),
    Pause,
    Stop,
    Seek(f64),
}

pub enum PlaybackEvent {
    Started,
    Paused,
    Stopped,
    Position(f64),
    Finished,
    Error(String),
}

pub struct AudioPlayer;

impl AudioPlayer {
    pub fn new() -> Self {
        Self
    }

    pub fn send_command(&self, _cmd: PlaybackCommand) -> Result<(), AppError> {
        Err(AppError::NotImplemented("send_command".to_string()))
    }

    pub fn try_recv_event(&self) -> Option<PlaybackEvent> {
        None
    }
}
