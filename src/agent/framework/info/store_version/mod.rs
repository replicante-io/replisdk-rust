//! Strategies to detect store version information.
use anyhow::Result;

use crate::agent::models::StoreVersion;
use crate::context::Context;

mod chain;
mod command;
mod file;
mod fixed;

pub use self::chain::StoreVersionChain;
pub use self::command::StoreVersionCommand;
pub use self::command::StoreVersionCommandConf;
pub use self::command::StoreVersionCommandError;
pub use self::file::StoreVersionFile;
pub use self::file::StoreVersionFileError;
pub use self::fixed::StoreVersionFixed;

/// Type of functions that can decode command outputs.
type DecodeFn = dyn Fn(Vec<u8>) -> Result<StoreVersion> + Send + Sync;

/// Interface to detect the version of the running store process.
#[async_trait::async_trait]
pub trait StoreVersionStrategy {
    /// Fetch the store version.
    async fn version(&self, context: &Context) -> Result<StoreVersion>;
}
