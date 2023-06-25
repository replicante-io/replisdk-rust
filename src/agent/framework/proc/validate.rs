//! Types to support agent process validation during initialisation.
use anyhow::Result;
use slog::Logger;

/// [`Validator`] that succeeds every time.
pub struct NoOpValidator {}

#[async_trait::async_trait]
impl Validator for NoOpValidator {
    async fn validate<'a>(&self, _: ValidatorArgs<'a>) -> Result<()> {
        Ok(())
    }
}

/// Hook to perform user validation as part of the agent initialisation.
#[async_trait::async_trait]
pub trait Validator {
    /// Execute custom validation logic.
    async fn validate<'a>(&self, args: ValidatorArgs<'a>) -> Result<()>;
}

/// Arguments passed to the agent validation logic.
pub struct ValidatorArgs<'a> {
    /// Configured logger for the process.
    pub logger: &'a Logger,
}
