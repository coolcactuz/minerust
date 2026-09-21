use crate::coords::ChunkPos;

/// Strongly-typed domain errors for world simulation and chunk persistence.
#[derive(thiserror::Error, Debug)]
pub enum WorldError {
    #[error("Chunk coordinate {0:?} is out of valid bounds")]
    OutOfBounds(ChunkPos),

    #[error("I/O failure during chunk file operation: {0}")]
    Io(#[from] std::io::Error),

    #[error("Chunk data corruption or invalid format: {0}")]
    Corruption(String),

    #[error("Chunk at {0:?} is not loaded in the active grid")]
    ChunkNotFound(ChunkPos),
}
