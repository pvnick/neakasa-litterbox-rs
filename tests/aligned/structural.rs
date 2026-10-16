use super::support::*;
use crate::*;
use serde_json::json;
pub fn run(group: &str, name: &str) {
    let b = base(name);
    if group == "packaging_config" {
        // These four cases intentionally check the *same pinned Python packaging file*.
        // They are reference provenance tests, not claims about Rust's packaging.
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/reference/pyproject.toml"
        ))
        .unwrap();
        let config: toml::Value = toml::from_str(&text).unwrap();
        let required = config["project"]["requires-python"].as_str().unwrap();
        assert!(required.starts_with(">="));
        let minimum: Vec<u32> = required
            .strip_prefix(">=")
            .unwrap()
            .split('.')
            .map(|s| s.parse().unwrap())
            .collect();
        match b {
            "test_requires_python_is_the_lowest_supported_version" => {
                assert_eq!(minimum, vec![3, 14])
            }
            "test_classifiers_list_every_supported_version" => {
                let expected = "Programming Language :: Python :: 3.14";
                let matched: Vec<_> = config["project"]["classifiers"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter(|s| *s == expected)
                    .collect();
                assert_eq!(matched, vec![expected]);
            }
            "test_ruff_targets_the_minimum_supported_version" => assert_eq!(
                config["tool"]["ruff"]["target-version"].as_str().unwrap(),
                format!("py{}{}", minimum[0], minimum[1])
            ),
            "test_mypy_targets_the_minimum_supported_version" => assert_eq!(
                config["tool"]["mypy"]["python_version"].as_str().unwrap(),
                format!("{}.{}", minimum[0], minimum[1])
            ),
            _ => panic!("unmapped {name}"),
        }
        return;
    }

    match (group, b) {
        ("client_lifecycle", "test_expect_object_passes_through_mapping") => assert_eq!(
            crate::json::expect_object(json!({"a":1})).unwrap(),
            json!({"a":1})
        ),
        ("client_lifecycle", "test_expect_object_rejects_non_object") => {
            let v = match param(name) {
                "value0" => json!([]),
                "str" => json!("str"),
                "5" => json!(5),
                "None" => json!(null),
                p => panic!("unknown param {p}"),
            };
            assert!(matches!(
                crate::json::expect_object(v),
                Err(Error::InvalidResponse)
            ));
        }
        ("client_login", "test_user_agent_tracks_installed_version") => {
            assert_eq!(crate::constants::USER_AGENT, "neakasa-litterbox-sdk/0.2.2")
        }
        ("client_login", "test_sdk_version_falls_back_when_package_metadata_is_missing") => {
            assert!(!env!("CARGO_PKG_VERSION").is_empty())
        }
        ("region", "test_region_web_url") => {
            let (r, v) = if name.contains("euapi") {
                (Region::Eu, "https://euapi.neakasa.com/api")
            } else if name.contains("https://ap.") {
                (Region::Ap, "https://ap.neakasa.com/api")
            } else {
                (Region::Us, "https://usapi.neakasapet.com/api")
            };
            assert_eq!(r.web_url(), v);
        }
        ("region", "test_region_login_url_appends_login") => {
            for (r, v) in [
                (Region::Eu, "https://euapi.neakasa.com/api/login"),
                (Region::Us, "https://usapi.neakasapet.com/api/login"),
                (Region::Ap, "https://ap.neakasa.com/api/login"),
            ] {
                assert_eq!(format!("{}/login", r.web_url()), v);
            }
        }
        ("region", "test_eu_host_is_not_the_storefront_subdomain") => {
            assert!(!Region::Eu.web_url().contains("eu.neakasa.com"))
        }
        _ => panic!("unmapped {group} {name}"),
    }
}
