//! The storage module of `DatenLord`.
//!
//! Designed by @xiaguan

#![allow(dead_code)] // TODO: Remove when this module is ready

mod backend;
mod block;
mod block_slice;
mod error;
mod handle;
mod memory_cache;
pub mod policy;
mod utils;

pub use backend::{Backend, BackendImpl};
pub use block::{Block, BLOCK_SIZE};
pub use block_slice::{offset_to_slice, BlockSlice};
pub use error::{StorageError, StorageResult};
pub use handle::handle::{FileHandle, Handles, OpenFlag};
pub use memory_cache::{CacheKey, MemoryCache};
pub use utils::{format_file_path, format_path};
