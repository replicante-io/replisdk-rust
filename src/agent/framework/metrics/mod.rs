//! Definitions of all Agent SDK metrics.
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::agent::framework::InitialiseHookArgs;

pub mod action;
pub mod store;

/// Register Agent SDK metrics during process initialisation.
pub fn initialise<C>(args: &InitialiseHookArgs<C>) -> Result<()>
where
    C: Clone + std::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
{
    let collectors: [Box<dyn prometheus::core::Collector>; 6] = [
        Box::new(action::EXECUTE_LOOPS_BUSY.clone()),
        Box::new(action::EXECUTE_LOOPS_DURATION.clone()),
        Box::new(action::EXECUTE_LOOPS_ERROR.clone()),
        Box::new(action::FAILED.clone()),
        Box::new(store::OPS_DURATION.clone()),
        Box::new(store::OPS_ERR.clone()),
    ];
    for collector in collectors {
        args.telemetry.metrics.register(collector)?;
    }
    Ok(())
}
