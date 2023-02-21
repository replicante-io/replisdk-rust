//! Utilities to manage store node templates in Replicante Platform servers.
use anyhow::Result;

/// A loaded store node template ready to be rendered.
///
/// # Experimental Properties
///
/// - Associated `Output` type: is this needed? Should it just be `String`?
/// - Should `Template`s be `Clone` too?
///   - Could limit implementations.
///   - But would allow caching `TemplateFactory` decorators and such.
///     - Could still do with generic type constraints instead of `Clone` super-trait?
pub trait Template {
    /// The output type of template rendering.
    type Output;

    /// Render the template with the provided context.
    fn render(&self, context: serde_json::Value) -> Result<Self::Output>;
}

/// Load templates from disk and prepares them for rendering.
///
/// Loading of templates includes initialisation of the templating engine with all extra
/// helpers, application globals and such possible features.
#[async_trait::async_trait]
pub trait TemplateFactory {
    /// The concrete [`Template`] type returned by this factory.
    type Template: Template;

    /// Load a template from disk.
    async fn load(&self, path: &std::path::Path) -> Result<Self::Template>;
}
