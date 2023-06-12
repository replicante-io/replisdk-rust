//! Execute a command to detect the store version.
use std::collections::BTreeMap;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tokio::process::Command;

use super::DecodeFn;
use super::StoreVersionStrategy;
use crate::agent::framework::DefaultContext;
use crate::agent::models::StoreVersion;

/// Execute a command to detect the store version.
///
/// By default the output is YAML decoded into a [`StoreVersion`] object
/// but a function can be provided to decode the command standard output in other ways.
pub struct StoreVersionCommand {
    args: Vec<String>,
    decoder: Option<Box<DecodeFn>>,
    env: BTreeMap<String, String>,
    exec: String,
}

impl StoreVersionCommand {
    /// Build a [`StoreVersionCommand`] that will use the provided command.
    pub fn build<S>(exec: S) -> StoreVersionCommandBuilder
    where
        S: Into<String>,
    {
        let command = StoreVersionCommand {
            args: Default::default(),
            decoder: None,
            env: Default::default(),
            exec: exec.into(),
        };
        StoreVersionCommandBuilder { command }
    }

    /// Build a [`StoreVersionCommand`] from a [`StoreVersionCommandConf`].
    pub fn with_conf(conf: StoreVersionCommandConf) -> StoreVersionCommandBuilder {
        let command = StoreVersionCommand {
            args: conf.args,
            decoder: None,
            env: conf.env,
            exec: conf.command,
        };
        StoreVersionCommandBuilder { command }
    }
}

#[async_trait::async_trait]
impl StoreVersionStrategy for StoreVersionCommand {
    async fn version(&self, context: &DefaultContext) -> Result<StoreVersion> {
        let mut command = Command::new(&self.exec);
        command.args(self.args.iter()).envs(self.env.iter());
        let output = command
            .output()
            .await
            .with_context(|| StoreVersionCommandError::CommandFailed(self.exec.clone()))?;

        // If the command fails debug log streams and fail.
        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr).unwrap_or_else(|_| "{binary}".into());
            let stdout = String::from_utf8(output.stdout).unwrap_or_else(|_| "{binary}".into());
            slog::debug!(
                context.logger, "Store version command failed";
                "stderr" => stderr,
                "stdout" => stdout,
            );
            anyhow::bail!(StoreVersionCommandError::CommandFailed(self.exec.clone()));
        }

        // Parse the process standard output to get the version.
        if let Some(decoder) = &self.decoder {
            return decoder(output.stdout).context(StoreVersionCommandError::Decode);
        }
        serde_yaml::from_slice(&output.stdout).context(StoreVersionCommandError::Decode)
    }
}

/// Build a [`StoreVersionCommand`].
pub struct StoreVersionCommandBuilder {
    command: StoreVersionCommand,
}

impl StoreVersionCommandBuilder {
    /// Append an argument to the command to execute.
    pub fn arg<S>(mut self, arg: S) -> Self
    where
        S: Into<String>,
    {
        self.command.args.push(arg.into());
        self
    }

    /// Set an output decoding function.
    pub fn decode<D>(mut self, decoder: D) -> Self
    where
        D: Fn(Vec<u8>) -> Result<StoreVersion> + Send + Sync + 'static,
    {
        let decoder = Box::new(decoder);
        self.command.decoder = Some(decoder);
        self
    }

    /// Complete creation of the command strategy.
    pub fn finish(self) -> StoreVersionCommand {
        self.command
    }
}

/// Configuration of the command to execute to detect the store version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoreVersionCommandConf {
    /// Arguments to pass to the command.
    #[serde(default)]
    pub args: Vec<String>,

    /// Name or path of the command to execute.
    pub command: String,

    /// Environment variables added for the command.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

/// Errors encountered while detecting the store version using a command.
#[derive(Debug, thiserror::Error)]
pub enum StoreVersionCommandError {
    /// Execution of the store version command failed.
    #[error("execution of the store version command failed: {0}")]
    // (executable,)
    CommandFailed(String),

    /// Unable to decode store version command output.
    #[error("unable to decode store version command output")]
    Decode,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::StoreVersionCommand;
    use super::StoreVersionStrategy;
    use crate::agent::models::StoreVersion;

    fn custom_decode(_: Vec<u8>) -> Result<StoreVersion> {
        Ok(StoreVersion {
            checkout: Some("ch".into()),
            number: "z.x.y".into(),
            extra: Some("ex".into()),
        })
    }

    #[tokio::test]
    async fn custom_decode_fn() {
        let context = crate::agent::framework::DefaultContext::fixture();
        let command = StoreVersionCommand::build("echo")
            .arg("custom")
            .decode(custom_decode)
            .finish();
        let version = command.version(&context).await.unwrap();
        assert_eq!(version.checkout, Some("ch".into()));
        assert_eq!(version.extra, Some("ex".into()));
        assert_eq!(version.number, "z.x.y".to_string());
    }

    #[tokio::test]
    async fn default_decode_yaml() {
        let context = crate::agent::framework::DefaultContext::fixture();
        let command = StoreVersionCommand::build("echo")
            .arg(r#"{"checkout": "c", "extra": "e", "number": "x.y.z"}"#)
            .finish();
        let version = command.version(&context).await.unwrap();
        assert_eq!(version.checkout, Some("c".into()));
        assert_eq!(version.extra, Some("e".into()));
        assert_eq!(version.number, "x.y.z".to_string());
    }
}
