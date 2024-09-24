use yabe::diff::{compute_diff, diff_and_common_multiple};
use yabe::deep_equal::deep_equal;
use yaml_rust2::{YamlLoader, Yaml};

#[test]
fn test_compute_diff_identical() {
    let helm_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();
    let obj_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();

    let diff = compute_diff(&obj_yaml, &helm_yaml);
    assert!(diff.is_none());
}

#[test]
fn test_compute_diff_simple_diff() {
    let helm_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();
    let obj_yaml = YamlLoader::load_from_str("a: 1\nb: 3").unwrap()[0].clone();

    let diff = compute_diff(&obj_yaml, &helm_yaml).unwrap();
    let expected_diff = YamlLoader::load_from_str("b: 3").unwrap()[0].clone();

    assert!(deep_equal(&diff, &expected_diff));
}

#[test]
fn test_compute_diff_nested() {
    let helm_yaml = YamlLoader::load_from_str("a:\n  b: 1\n  c: 2").unwrap()[0].clone();
    let obj_yaml = YamlLoader::load_from_str("a:\n  b: 1\n  c: 3").unwrap()[0].clone();

    let diff = compute_diff(&obj_yaml, &helm_yaml).unwrap();
    let expected_diff = YamlLoader::load_from_str("a:\n  c: 3").unwrap()[0].clone();

    assert!(deep_equal(&diff, &expected_diff));
}

#[test]
fn test_compute_diff_array() {
    let helm_yaml = YamlLoader::load_from_str("items:\n  - a\n  - b").unwrap()[0].clone();
    let obj_yaml = YamlLoader::load_from_str("items:\n  - a\n  - c").unwrap()[0].clone();

    let diff = compute_diff(&obj_yaml, &helm_yaml).unwrap();
    let expected_diff = YamlLoader::load_from_str("items:\n  - null\n  - c").unwrap()[0].clone();

    assert!(deep_equal(&diff, &expected_diff));
}

#[test]
fn test_compute_diff_additional_key() {
    let helm_yaml = YamlLoader::load_from_str("a: 1").unwrap()[0].clone();
    let obj_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();

    let diff = compute_diff(&obj_yaml, &helm_yaml).unwrap();
    let expected_diff = YamlLoader::load_from_str("b: 2").unwrap()[0].clone();

    assert!(deep_equal(&diff, &expected_diff));
}

#[test]
fn test_compute_diff_missing_key() {
    let helm_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();
    let obj_yaml = YamlLoader::load_from_str("a: 1").unwrap()[0].clone();

    let diff = compute_diff(&obj_yaml, &helm_yaml);
    // Since obj_yaml lacks 'b', and compute_diff only considers keys in obj_yaml,
    // the diff should be None (no differences).
    assert!(diff.is_none());
}

#[test]
fn test_diff_and_common_multiple_identical() {
    let yaml1 = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2];

    let (base, diffs) = diff_and_common_multiple(&objs, None);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();
    assert!(deep_equal(&base.unwrap(), &expected_base));

    assert!(diffs.iter().all(|d| d.is_none()));
}

#[test]
fn test_diff_and_common_multiple_different() {
    let yaml1 = YamlLoader::load_from_str("a: 1\nb: 2").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("a: 1\nb: 3").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2];

    let (base, diffs) = diff_and_common_multiple(&objs, None);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("a: 1").unwrap()[0].clone();
    assert!(deep_equal(&base.unwrap(), &expected_base));

    let expected_diffs = vec![
        YamlLoader::load_from_str("b: 2").unwrap()[0].clone(),
        YamlLoader::load_from_str("b: 3").unwrap()[0].clone(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap(), expected_diff));
    }
}

#[test]
fn test_diff_and_common_multiple_three_files() {
    let yaml1 = YamlLoader::load_from_str("a: 1\nb: 2\nc: 3").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("a: 1\nb: 2\nc: 4").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("a: 1\nb: 5\nc: 3").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let (base, diffs) = diff_and_common_multiple(&objs, None);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("a: 1").unwrap()[0].clone();
    assert!(deep_equal(&base.unwrap(), &expected_base));

    let expected_diffs = vec![
        YamlLoader::load_from_str("b: 2\nc: 3").unwrap()[0].clone(),
        YamlLoader::load_from_str("b: 2\nc: 4").unwrap()[0].clone(),
        YamlLoader::load_from_str("b: 5\nc: 3").unwrap()[0].clone(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap(), expected_diff));
    }
}

#[test]
fn test_diff_and_common_multiple_arrays() {
    let yaml1 = YamlLoader::load_from_str("items:\n  - a\n  - b").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("items:\n  - a\n  - c").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2];

    let (base, diffs) = diff_and_common_multiple(&objs, None);

    // Since arrays differ, base is None
    assert!(base.is_none());

    let expected_diffs = vec![
        YamlLoader::load_from_str("items:\n  - a\n  - b").unwrap()[0].clone(),
        YamlLoader::load_from_str("items:\n  - a\n  - c").unwrap()[0].clone(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap(), expected_diff));
    }
}