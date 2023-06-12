//! Return a fixed store version every time.
use anyhow::Result;

use super::StoreVersionStrategy;
use crate::agent::framework::DefaultContext;
use crate::agent::models::StoreVersion;

/// Return a fixed store version every time.
pub struct StoreVersionFixed {
    version: StoreVersion,
}

impl StoreVersionFixed {
    /// Create a strategy to return the given store version.
    pub fn new(version: StoreVersion) -> StoreVersionFixed {
        StoreVersionFixed { version }
    }
}

#[async_trait::async_trait]
impl StoreVersionStrategy for StoreVersionFixed {
    async fn version(&self, _: &DefaultContext) -> Result<StoreVersion> {
        Ok(self.version.clone())
    }
}
