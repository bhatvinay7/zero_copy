use std::fmt;

#[derive(Debug)]
pub enum TranscoderError {
    Io(std::io::Error),
    Anyhow(anyhow::Error),
    Lapin(lapin::Error),
    Redis(redis::RedisError),
    JobCancelled,
}

impl std::error::Error for TranscoderError {}

impl fmt::Display for TranscoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::Anyhow(err) => write!(f, "Database/network error: {}", err),
            Self::Lapin(err) => write!(f, "RabbitMQ error: {}", err),
            Self::Redis(err) => write!(f, "Redis error: {}", err),
            Self::JobCancelled => write!(f, "Transcode job was cancelled by user"),
        }
    }
}

impl From<std::io::Error> for TranscoderError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<anyhow::Error> for TranscoderError {
    fn from(err: anyhow::Error) -> Self {
        Self::Anyhow(err)
    }
}

impl From<lapin::Error> for TranscoderError {
    fn from(err: lapin::Error) -> Self {
        Self::Lapin(err)
    }
}

impl From<redis::RedisError> for TranscoderError {
    fn from(err: redis::RedisError) -> Self {
        Self::Redis(err)
    }
}
