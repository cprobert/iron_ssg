use iron_ssg::IronSSGSiteManifest;
use toml;

#[test]
fn test_toml_deserialization() {
    use super::*;
    use toml;

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
public = ["public", "_generated"]
"#;

        let config: Result<IronSSGConfig, toml::de::Error> = toml::from_str(toml_str);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(
            config.public,
            Some(vec!["public".to_string(), "_generated".to_string()])
        );
    }
}
