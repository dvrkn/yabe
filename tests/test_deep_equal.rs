use yabe::deep_equal::deep_equal;
use yaml_rust2::{YamlLoader, Yaml};

#[test]
fn test_deep_equal_scalars() {
    // Integers
    assert!(deep_equal(&Yaml::Integer(42), &Yaml::Integer(42)));
    assert!(!deep_equal(&Yaml::Integer(42), &Yaml::Integer(43)));

    // Strings
    assert!(deep_equal(&Yaml::String("hello".into()), &Yaml::String("hello".into())));
    assert!(!deep_equal(&Yaml::String("hello".into()), &Yaml::String("world".into())));

    // Booleans
    assert!(deep_equal(&Yaml::Boolean(true), &Yaml::Boolean(true)));
    assert!(!deep_equal(&Yaml::Boolean(true), &Yaml::Boolean(false)));

    // Null
    assert!(deep_equal(&Yaml::Null, &Yaml::Null));
    assert!(!deep_equal(&Yaml::Null, &Yaml::Integer(0)));
}

#[test]
fn test_deep_equal_arrays() {
    let yaml1 = YamlLoader::load_from_str("- 1\n- 2\n- 3").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("- 1\n- 2\n- 3").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("- 1\n- 2\n- 4").unwrap()[0].clone();

    assert!(deep_equal(&yaml1, &yaml2));
    assert!(!deep_equal(&yaml1, &yaml3));
}

#[test]
fn test_deep_equal_hashes() {
    let yaml1 = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("a: 1\nb: 3").unwrap()[0].clone();

    assert!(deep_equal(&yaml1, &yaml2));
    assert!(!deep_equal(&yaml1, &yaml3));
}

#[test]
fn test_deep_equal_nested() {
    let yaml1 = YamlLoader::load_from_str("a:\n  b: 1\n  c:\n    - x\n    - y").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("a:\n  b: 1\n  c:\n    - x\n    - y").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("a:\n  b: 2\n  c:\n    - x\n    - z").unwrap()[0].clone();

    assert!(deep_equal(&yaml1, &yaml2));
    assert!(!deep_equal(&yaml1, &yaml3));
}
