use std::hash::Hash;
use yabe::diff::{compute_diff, diff_and_common_multiple};
use yabe::deep_equal::deep_equal;
use yaml_rust2::{Yaml, YamlLoader};

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

    let (base, diffs) = diff_and_common_multiple(&objs, 0.51);

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

    let (base, diffs) = diff_and_common_multiple(&objs, 0.51);

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
fn test_diff_and_common_multiple_three_files_quorum_66() {
    let yaml1 = YamlLoader::load_from_str("a: 1\nb: 2\nc: 3").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("a: 1\nb: 2\nc: 4").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("a: 1\nb: 5\nc: 3").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let quorum_percentage = 0.66; // 66%
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("a: 1\nb: 2\nc: 3").unwrap()[0].clone();
    assert!(deep_equal(&base.unwrap(), &expected_base));

    let expected_diffs = vec![
        None, // yaml1 matches the base
        Some(YamlLoader::load_from_str("c: 4").unwrap()[0].clone()),
        Some(YamlLoader::load_from_str("b: 5").unwrap()[0].clone()),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        match expected_diff {
            Some(expected) => {
                assert!(diff.is_some());
                assert!(deep_equal(diff.as_ref().unwrap(), expected));
            }
            None => assert!(diff.is_none()),
        }
    }
}

#[test]
fn test_diff_and_common_multiple_arrays() {
    let yaml1 = YamlLoader::load_from_str("items:\n  - a\n  - b").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("items:\n  - a\n  - c").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2];

    let (base, diffs) = diff_and_common_multiple(&objs, 0.51);

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

#[test]
fn test_quorum_100_percent() {
    let yaml1 = YamlLoader::load_from_str("key1: value1\nkey2: value2").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("key1: value1\nkey2: value3").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("key1: value1\nkey2: value2").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let quorum_percentage = 1.0; // 100%
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    // Only key1 should be in the base
    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("key1: value1").unwrap()[0].clone();
    assert!(deep_equal(&base.unwrap(), &expected_base));

    // Diffs should contain deviations in key2
    let expected_diffs = vec![
        YamlLoader::load_from_str("key2: value2").unwrap()[0].clone(),
        YamlLoader::load_from_str("key2: value3").unwrap()[0].clone(),
        YamlLoader::load_from_str("key2: value2").unwrap()[0].clone(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap(), expected_diff));
    }
}

#[test]
fn test_quorum_66_percent() {
    let yaml1 = YamlLoader::load_from_str("key1: value_common\nkey2: value1").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("key1: value_common\nkey2: value2").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("key1: value_common\nkey2: value1").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let quorum_percentage = 0.66; // 66%
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    // key1 and key2 with value1 should be in the base
    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("key1: value_common\nkey2: value1").unwrap()[0].clone();
    assert!(deep_equal(&base.unwrap(), &expected_base));

    // Diffs should contain deviations in key2 for yaml2
    let expected_diffs = vec![
        None, // yaml1 matches the base
        Some(YamlLoader::load_from_str("key2: value2").unwrap()[0].clone()),
        None, // yaml3 matches the base
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        match expected_diff {
            Some(expected) => {
                assert!(diff.is_some());
                assert!(deep_equal(diff.as_ref().unwrap(), expected));
            }
            None => assert!(diff.is_none()),
        }
    }
}

#[test]
fn test_quorum_50_percent() {
    let yaml1 = YamlLoader::load_from_str("key1: value_common\nkey2: value1").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("key1: value_common\nkey2: value2").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("key1: value_common\nkey2: value3").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let quorum_percentage = 0.5; // 50%
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    // Only key1 should be in the base
    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("key1: value_common").unwrap()[0].clone();
    assert!(deep_equal(&base.unwrap(), &expected_base));

    // Diffs should contain deviations in key2 for each file
    let expected_diffs = vec![
        YamlLoader::load_from_str("key2: value1").unwrap()[0].clone(),
        YamlLoader::load_from_str("key2: value2").unwrap()[0].clone(),
        YamlLoader::load_from_str("key2: value3").unwrap()[0].clone(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap(), expected_diff));
    }
}

#[test]
fn test_quorum_0_percent() {
    let yaml1 = YamlLoader::load_from_str("key1: value1").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("key1: value2").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("key1: value3").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let quorum_percentage = 0.0; // 0%
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    // Since quorum is 0%, any value is acceptable as base
    assert!(base.is_some());
    let base_yaml = base.unwrap();
    let type_of_base = std::any::type_name::<Yaml>();
    assert_eq!(type_of_base, "yaml_rust2::yaml::Yaml");

    // One random value should be in the base, and the rest should be in diffs
    let filtered_diffs = diffs.iter().filter(|x| x.is_some()).collect::<Vec<_>>();
    assert_eq!(filtered_diffs.len(), 2);
}

#[test]
fn test_quorum_edge_case_high_quorum() {
    let yaml1 = YamlLoader::load_from_str("key1: value1\nkey2: value_common").unwrap()[0].clone();
    let yaml2 = YamlLoader::load_from_str("key1: value2\nkey2: value_common").unwrap()[0].clone();
    let yaml3 = YamlLoader::load_from_str("key1: value3\nkey2: value_common").unwrap()[0].clone();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let quorum_percentage = 0.9; // 90%
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    // key2 meets the quorum (100%), key1 does not
    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("key2: value_common").unwrap()[0].clone();
    assert!(deep_equal(&base.unwrap(), &expected_base));

    // Diffs should include key1 for each file
    let expected_diffs = vec![
        YamlLoader::load_from_str("key1: value1").unwrap()[0].clone(),
        YamlLoader::load_from_str("key1: value2").unwrap()[0].clone(),
        YamlLoader::load_from_str("key1: value3").unwrap()[0].clone(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap(), expected_diff));
    }
}
