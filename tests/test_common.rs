use yabe::diff::{compute_diff, diff_and_common_multiple};
use yabe::deep_equal::deep_equal;
use yaml_rust2::{YamlLoader, Yaml};

#[test]
fn test_empty_documents() {
    let yaml1 = YamlLoader::load_from_str("").unwrap_or_default().get(0).cloned().unwrap_or(Yaml::Null);
    let yaml2 = YamlLoader::load_from_str("").unwrap_or_default().get(0).cloned().unwrap_or(Yaml::Null);

    assert!(deep_equal(&yaml1, &yaml2));

    let diff = compute_diff(&yaml1, &yaml2);
    assert!(diff.is_none());

    let objs = vec![&yaml1, &yaml2];
    let (base, diffs) = diff_and_common_multiple(&objs, None);

    assert!(base.is_some());
    assert!(deep_equal(&base.unwrap(), &Yaml::Null));
    assert!(diffs.iter().all(|d| d.is_none()));
}

#[test]
fn test_different_types_same_key() {
    let yaml1 = YamlLoader::load_from_str("key: value").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("key:\n  subkey: value").unwrap()[0].clone();

    let objs = vec![&yaml1, &yaml2];
    let (base, diffs) = diff_and_common_multiple(&objs, None);

    assert!(base.is_none());

    let expected_diffs = vec![
        YamlLoader::load_from_str("key: value").unwrap()[0].clone(),
        YamlLoader::load_from_str("key:\n  subkey: value").unwrap()[0].clone(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap(), expected_diff));
    }
}

#[test]
fn test_null_values() {
    let yaml1 = YamlLoader::load_from_str("a: null").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("a: value").unwrap()[0].clone();

    let diff = compute_diff(&yaml1, &yaml2).unwrap();
    let expected_diff = YamlLoader::load_from_str("a: null").unwrap()[0].clone();

    assert!(deep_equal(&diff, &expected_diff));

    let objs = vec![&yaml1, &yaml2];
    let (base, diffs) = diff_and_common_multiple(&objs, None);

    assert!(base.is_none());

    let expected_diffs = vec![
        YamlLoader::load_from_str("a: null").unwrap()[0].clone(),
        YamlLoader::load_from_str("a: value").unwrap()[0].clone(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap(), expected_diff));
    }
}