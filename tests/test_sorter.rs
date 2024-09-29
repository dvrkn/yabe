use yaml_rust2::{YamlLoader, Yaml, YamlEmitter};
use std::borrow::Cow;
use std::fs;
use log::warn;
use yabe::sorter::sort_yaml;

// return yaml or error
pub fn init_test_config(config_path: &str) -> Yaml {
    let config_str = fs::read_to_string(config_path).unwrap_or_else(|e| {
        warn!("Failed to read config file: {}", e);
        String::new()
    });

    YamlLoader::load_from_str(&config_str).unwrap_or_else(|e| {
        warn!("Failed to parse config file: {}", e);
        vec![]
    }).into_iter().next().unwrap_or(Yaml::Null)
}

#[test]
fn test_load_config() {
    let config = init_test_config("tests/config.yaml");

    assert!(config["preOrder"].is_array());
    assert_eq!(config["sortKey"].as_str().unwrap(), "name");
}

#[test]
fn test_hash_sorter() {
    let config = init_test_config("tests/config.yaml");

    let test_str = r#"
        c: 3
        b: 2
        a: 1
        "#;

    let docs = YamlLoader::load_from_str(test_str).unwrap();
    let doc = &docs[0];
    let processed_doc = sort_yaml(Cow::Borrowed(doc), &config);
    println!("{:?}", processed_doc);
    assert_eq!(processed_doc.into_owned(), Yaml::Hash(
        vec![
            (Yaml::String("a".to_string()), Yaml::Integer(1)),
            (Yaml::String("b".to_string()), Yaml::Integer(2)),
            (Yaml::String("c".to_string()), Yaml::Integer(3)),
        ].into_iter().collect()
    ));
}

#[test]
fn test_array_sorter() {
    let config = init_test_config("tests/config.yaml");
    let test_str = r#"
        - name: Bob
        - name: Alice
        - name: Carol
        "#;

    let docs = YamlLoader::load_from_str(test_str).unwrap();
    let doc = &docs[0];
    let processed_doc = sort_yaml(Cow::Borrowed(doc), &config);
    assert_eq!(processed_doc.into_owned(), Yaml::Array(
        vec![
            Yaml::Hash(
                vec![(Yaml::String("name".to_string()), Yaml::String("Alice".to_string()))].into_iter().collect()
            ),
            Yaml::Hash(
                vec![(Yaml::String("name".to_string()), Yaml::String("Bob".to_string()))].into_iter().collect()
            ),
            Yaml::Hash(
                vec![(Yaml::String("name".to_string()), Yaml::String("Carol".to_string()))].into_iter().collect()
            ),
        ]
    ));
}

#[test]
fn full_test() {
    let config = init_test_config("config-gitops.yaml");

    let test_str = r#"
test: yaml
namespace: argocd
name: test
apiVersion: argoproj.io/v1alpha1
arr:
  - test: without sort key
  - test: yaml
    namespace: argocd
    name: test
    apiVersion: argoproj.io/v1alpha1
    arr2:
      - namespace: argocd
        name: test
        apiVersion: argoproj.io/v1alpha1
        test: yaml
        enabled: false
  - enabled: false
    name: arr
enabled: false
spec:
  generators:
    - list:
        elements:
          - name: c
          - name: b
          - name: a
            ord:
              - name: c
              - name: b
              - name: a

  name: cluster-resources

# Comment
anchor: &test
  - anchor
anchor-test: *test
    "#;

    let result = r#"---
enabled: false
apiVersion: argoproj.io/v1alpha1
name: test
namespace: argocd
spec:
  name: cluster-resources
  generators:
    - list:
        elements:
          - name: a
            ord:
              - name: a
              - name: b
              - name: c
          - name: b
          - name: c
anchor:
  - anchor
anchor-test:
  - anchor
arr:
  - enabled: false
    name: arr
  - apiVersion: argoproj.io/v1alpha1
    name: test
    namespace: argocd
    arr2:
      - enabled: false
        apiVersion: argoproj.io/v1alpha1
        name: test
        namespace: argocd
        test: yaml
    test: yaml
  - test: without sort key
test: yaml"#;

    let docs = YamlLoader::load_from_str(test_str).unwrap();
    let doc = &docs[0];
    let processed_doc = sort_yaml(Cow::Borrowed(doc), &config);

    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&processed_doc).unwrap();
    }

    assert_eq!(out_str, result);
}