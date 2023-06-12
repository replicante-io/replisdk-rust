//! Read the store version from a file.
use anyhow::Context;
use anyhow::Result;

use super::DecodeFn;
use super::StoreVersionStrategy;
use crate::agent::framework::DefaultContext;
use crate::agent::models::StoreVersion;

/// Read the store version from a file.
///
/// The file is read every time the version is checked to ensure
/// changes are picked up.
pub struct StoreVersionFile {
    decoder: Option<Box<DecodeFn>>,
    path: String,
}

impl StoreVersionFile {
    /// Set an output decoding function.
    pub fn decode<D>(mut self, decoder: D) -> Self
    where
        D: Fn(Vec<u8>) -> Result<StoreVersion> + Send + Sync + 'static,
    {
        let decoder = Box::new(decoder);
        self.decoder = Some(decoder);
        self
    }

    /// Build a [`StoreVersionFile`] that will read the given file.
    pub fn new<P>(path: P) -> StoreVersionFile
    where
        P: Into<String>,
    {
        StoreVersionFile {
            decoder: None,
            path: path.into(),
        }
    }
}

#[async_trait::async_trait]
impl StoreVersionStrategy for StoreVersionFile {
    async fn version(&self, _: &DefaultContext) -> Result<StoreVersion> {
        // Read the whole file into a buffer.
        let data = tokio::fs::read(&self.path)
            .await
            .with_context(|| StoreVersionFileError::Io(self.path.clone()))?;

        // Decode the buffer with the given function (or as yaml otherwise).
        if let Some(decoder) = &self.decoder {
            return decoder(data).context(StoreVersionFileError::Decode);
        }
        serde_yaml::from_slice(&data).context(StoreVersionFileError::Decode)
    }
}

/// Errors encountered while detecting the store version from a file.
#[derive(Debug, thiserror::Error)]
pub enum StoreVersionFileError {
    /// Unable to decode store version from file.
    #[error("unable to decode store version from file")]
    Decode,

    /// Unable to read the version file.
    #[error("Unable to read the version file '{0}'")]
    // (path,)
    Io(String),
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::StoreVersionFile;
    use super::StoreVersionFileError;
    use super::StoreVersionStrategy;
    use crate::agent::framework::DefaultContext;
    use crate::agent::models::StoreVersion;

    const TEST_CUSTOM: &str = "src/agent/framework/info/store_version/fixtures/file_custom.txt";
    const TEST_DEFAULT: &str = "src/agent/framework/info/store_version/fixtures/file_default.yaml";
    const TEST_NOT_FINED: &str = "/tmp/replitest/path/does/not/exists";

    fn custom_decode(_: Vec<u8>) -> Result<StoreVersion> {
        Ok(StoreVersion {
            checkout: Some("ch".into()),
            number: "z.x.y".into(),
            extra: Some("ex".into()),
        })
    }

    #[tokio::test]
    async fn file_not_found() {
        let strategy = StoreVersionFile::new(TEST_NOT_FINED);
        let context = DefaultContext::fixture();
        let version = strategy.version(&context).await;
        match version {
            Ok(version) => panic!("expected StoreVersionFileError, got version {:?}", version),
            Err(error) if error.is::<StoreVersionFileError>() => (),
            Err(error) => panic!("expected StoreVersionFileError, got error {:?}", error),
        }
    }

    #[tokio::test]
    async fn custom_decode_fn() {
        let strategy = StoreVersionFile::new(TEST_CUSTOM).decode(custom_decode);
        let context = DefaultContext::fixture();
        let version = strategy.version(&context).await.unwrap();
        assert_eq!(version.checkout, Some("ch".into()));
        assert_eq!(version.extra, Some("ex".into()));
        assert_eq!(version.number, "z.x.y".to_string());
    }

    #[tokio::test]
    async fn default_decode_yaml() {
        let strategy = StoreVersionFile::new(TEST_DEFAULT);
        let context = DefaultContext::fixture();
        let version = strategy.version(&context).await.unwrap();
        assert_eq!(version.checkout, Some("c".into()));
        assert_eq!(version.extra, Some("e".into()));
        assert_eq!(version.number, "x.y.z".to_string());
    }
}
