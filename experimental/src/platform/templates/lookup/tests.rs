use anyhow::Result;

use super::TemplateLoadOptions;
use super::TemplateLookup;
use crate::platform::templates::TemplateFactory;

struct RuleFactory();

#[async_trait::async_trait]
impl TemplateFactory for RuleFactory {
    type Template = TemplateLoadOptions;

    async fn load(&self, options: &TemplateLoadOptions) -> Result<Self::Template> {
        Ok(options.clone())
    }
}

#[tokio::test]
async fn load_manifests() {
    let templates = TemplateLookup::load_file(
        RuleFactory(),
        "src/platform/templates/lookup/fixtures/stores.yaml",
    )
    .await
    .unwrap();
    assert_eq!(templates.stores.len(), 2);

    let rule = &templates.stores[0];
    assert_eq!(rule.store, "test.simple.store");
}

#[tokio::test]
async fn load_manifests_many() {
    let mut templates = TemplateLookup::load_file(
        RuleFactory(),
        "src/platform/templates/lookup/fixtures/stores.yaml",
    )
    .await
    .unwrap();
    templates
        .extend_from_file("src/platform/templates/lookup/fixtures/stores.extended.yaml")
        .await
        .unwrap();
    assert_eq!(templates.stores.len(), 3);

    let rule = &templates.stores[0];
    assert_eq!(rule.store, "test.simple.store");
    let rule = &templates.stores[2];
    assert_eq!(rule.store, "postgres.2");
}

#[tokio::test]
async fn lookup_template() {
    let templates = TemplateLookup::load_file(
        RuleFactory(),
        "src/platform/templates/lookup/fixtures/stores.yaml",
    )
    .await
    .unwrap();
    let attributes = {
        let mut attrs = serde_json::Map::new();
        attrs.insert("store.matched".into(), 42.into());
        attrs.insert("version.matched".into(), "yup".into());
        attrs
    };
    let context = crate::platform::templates::TemplateContext {
        attributes,
        cluster_id: "WHO_CARES".into(),
        store: "postgres".into(),
        store_version: "1.2.3".into(),
    };
    let template = templates.lookup(&context).await.unwrap().unwrap();
    assert_eq!(
        template.template,
        "src/platform/templates/lookup/fixtures/version/selected/by/lookup",
    );
}

mod attributes_match {
    use super::super::attributes_match;

    #[test]
    fn no_attrs_no_matchers() {
        let attributes = serde_json::Map::new();
        let matchers = Default::default();
        let did_match = attributes_match(&attributes, &matchers);
        assert_eq!(did_match, true);
    }

    #[test]
    fn no_attrs_with_matchers() {
        let attributes = serde_json::Map::new();
        let matchers = {
            let mut matchers = std::collections::HashMap::default();
            matchers.insert("mode".into(), "none".into());
            matchers
        };
        let did_match = attributes_match(&attributes, &matchers);
        assert_eq!(did_match, false);
    }

    #[test]
    fn with_attrs_no_matchers() {
        let attributes = {
            let mut attrs = serde_json::Map::new();
            attrs.insert("mode".into(), "none".into());
            attrs
        };
        let matchers = Default::default();
        let did_match = attributes_match(&attributes, &matchers);
        assert_eq!(did_match, true);
    }

    #[test]
    fn with_attrs_with_matchers_diff() {
        let attributes = {
            let mut attrs = serde_json::Map::new();
            attrs.insert("mode".into(), "some".into());
            attrs
        };
        let matchers = {
            let mut matchers = std::collections::HashMap::default();
            matchers.insert("mode".into(), "none".into());
            matchers
        };
        let did_match = attributes_match(&attributes, &matchers);
        assert_eq!(did_match, false);
    }

    #[test]
    fn with_attrs_with_matchers_same() {
        let attributes = {
            let mut attrs = serde_json::Map::new();
            attrs.insert("mode".into(), "none".into());
            attrs
        };
        let matchers = {
            let mut matchers = std::collections::HashMap::default();
            matchers.insert("mode".into(), "none".into());
            matchers
        };
        let did_match = attributes_match(&attributes, &matchers);
        assert_eq!(did_match, true);
    }
}
