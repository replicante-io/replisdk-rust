//! The [`Context`] is a general purpose container to carry scoped values around.
//!
//! Other frameworks provided by this SDK use [`Context`] to handle access to
//! per-request scoped data.
//!
//! Contexts are organised into a tree structure:
//!
//! - A root context represents most generic scope (usually the entire process).
//! - Derived contexts represents a narrower scope within their parent with additional
//!   or updated information attached to them.
//!
//! For example: [`Context`]s provide access to the current [`Logger`].
//! For the root context this is the process-wide logger with no additional attributes.
//! But for individual operations a derived context can be provided with a [`Logger`] decorated
//! with the operation trace ID or other request attributes.
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use slog::Logger;
use slog::OwnedKV;
use slog::SendSyncRefUnwindSafeKV;

#[cfg(feature = "actix-web")]
mod actix;
#[cfg(any(feature = "opentelemetry", feature = "opentelemetry_api"))]
mod otel;

#[cfg(feature = "actix-web")]
pub use {self::actix::ActixMiddleware, self::actix::ActixTransform, self::actix::ContextConfig};

/// The [`Context`] is a general purpose container to carry scoped values around.
///
/// Refer to the [module level docs](crate::context) for details.
#[derive(Clone, Debug)]
pub struct Context {
    /// Logger with contextual attributes attached to it.
    pub logger: Logger,

    /// Store arbitrary data attached to the context.
    entries: HashMap<TypeId, Arc<dyn Any + Sync + Send>>,
}

impl Context {
    /// Derive a new [`Context`] by making changes to the current one.
    pub fn derive(&self) -> ContextBuilder {
        ContextBuilder {
            entries: self.entries.clone(),
            logger: self.logger.clone(),
        }
    }

    /// Derive a new [`Context`] by making changes to the current one using the provided callback.
    pub fn derive_with<F>(&self, callback: F) -> Context
    where
        F: FnOnce(ContextBuilder) -> ContextBuilder,
    {
        let builder = callback(self.derive());
        builder.build()
    }

    /// Retrieve a custom value by type from the context.
    ///
    /// ## Panics
    ///
    /// This method panics with the given message if the specified type does not have a value.
    pub fn expect<T>(&self, msg: &str) -> &T
    where
        T: 'static + Send + Sync,
    {
        self.get::<T>().expect(msg)
    }

    /// Retrieve a custom value by type from the context.
    pub fn get<T>(&self) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        self.entries
            .get(&TypeId::of::<T>())
            .and_then(|entry| entry.downcast_ref())
    }

    /// Retrieve a custom value by type from the context.
    ///
    /// ## Panics
    ///
    /// This method panics with the given message if the specified type does not have a value.
    pub fn require<T>(&self) -> &T
    where
        T: 'static + Send + Sync,
    {
        self.expect::<T>("context does not hold a value for the required type")
    }

    /// Initialise a new root context with no values attached.
    pub fn root(logger: Logger) -> ContextBuilder {
        ContextBuilder {
            entries: Default::default(),
            logger,
        }
    }
}

#[cfg(any(test, feature = "test-fixture"))]
impl Context {
    /// Create an empty context useful for test.
    pub fn fixture() -> Context {
        let logger = Logger::root(slog::Discard, slog::o!());
        Context {
            logger,
            entries: Default::default(),
        }
    }
}

/// A builder for root and derived contexts.
pub struct ContextBuilder {
    entries: HashMap<TypeId, Arc<dyn Any + Sync + Send>>,
    logger: Logger,
}

impl ContextBuilder {
    /// Finalise the build process and return a new [`Context`].
    pub fn build(self) -> Context {
        Context {
            logger: self.logger,
            entries: self.entries,
        }
    }

    /// Update the [`Context`] logger to attach new log key/pair values.
    pub fn log_values<T>(mut self, entries: OwnedKV<T>) -> Self
    where
        T: SendSyncRefUnwindSafeKV + 'static,
    {
        self.logger = self.logger.new(entries);
        self
    }

    /// Attach a value to the context.
    pub fn value<T>(mut self, value: T) -> Self
    where
        T: 'static + Send + Sync,
    {
        self.entries.insert(TypeId::of::<T>(), Arc::new(value));
        self
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::sync::Arc;

    use super::Context;

    #[test]
    fn derive_log_attributes() {
        let root = Context::fixture();
        let parent = root
            .derive()
            .log_values(slog::o!("root" => "value", "test" => "root"))
            .build();
        let context = parent
            .derive()
            .log_values(slog::o!("test" => "override"))
            .build();
        assert_eq!(format!("{:?}", context.logger.list()), "(test, test, root)");
    }

    #[test]
    fn derive_noop() {
        let parent = Context::fixture();
        let context = parent.derive().build();
        assert_eq!(
            format!("{:?}", parent.logger.list()),
            format!("{:?}", context.logger.list()),
        );
    }

    #[test]
    fn extra_expect_with() {
        let mut context = Context::fixture();
        context.entries.insert(TypeId::of::<u64>(), Arc::new(42u64));
        let value = context.expect::<u64>("test to pass");
        assert_eq!(value, &42);
    }

    #[test]
    #[should_panic(expected = "extract should panic")]
    fn extra_expect_without() {
        let context = Context::fixture();
        context.expect::<u64>("extract should panic");
    }

    #[test]
    fn extra_get_with() {
        let mut context = Context::fixture();
        context.entries.insert(TypeId::of::<u64>(), Arc::new(42u64));
        let value = context.get::<u64>();
        assert_eq!(value, Some(&42));
    }

    #[test]
    fn extra_get_without() {
        let context = Context::fixture();
        let value = context.get::<u64>();
        assert_eq!(value, None);
    }

    #[test]
    fn extra_require_with() {
        let mut context = Context::fixture();
        context.entries.insert(TypeId::of::<u64>(), Arc::new(42u64));
        let value = context.require::<u64>();
        assert_eq!(value, &42);
    }

    #[test]
    #[should_panic(expected = "context does not hold a value for the required type")]
    fn extra_require_without() {
        let context = Context::fixture();
        context.require::<u64>();
    }
}
