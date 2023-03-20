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
    let attributes = serde_json::json!({
        "store.matched": 42,
        "version.matched": "yup",
    });
    let version = semver::Version::parse("1.2.3").unwrap();
    let template = templates
        .lookup("postgres", &version, &attributes)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(template.template, "version/selected/by/lookup");
}

mod attributes_match {
    use super::super::attributes_match;

    #[test]
    fn no_attrs_no_matchers() {
        let attributes = serde_json::json!({});
        let matchers = Default::default();
        let did_match = attributes_match(&attributes, &matchers);
        assert_eq!(did_match, true);
    }

    #[test]
    fn no_attrs_with_matchers() {
        let attributes = serde_json::json!({});
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
        let attributes = serde_json::json!({
            "mode": "none",
        });
        let matchers = Default::default();
        let did_match = attributes_match(&attributes, &matchers);
        assert_eq!(did_match, true);
    }

    #[test]
    fn with_attrs_with_matchers_diff() {
        let attributes = serde_json::json!({
            "mode": "some",
        });
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
        let attributes = serde_json::json!({
            "mode": "none",
        });
        let matchers = {
            let mut matchers = std::collections::HashMap::default();
            matchers.insert("mode".into(), "none".into());
            matchers
        };
        let did_match = attributes_match(&attributes, &matchers);
        assert_eq!(did_match, true);
    }
}
