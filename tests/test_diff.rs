use yabe::diff::{compute_diff, diff_and_common_multiple};
use yabe::deep_equal::deep_equal;
use yaml_rust2::YamlLoader;

#[test]
fn test_compute_diff_identical() {
    let helm_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap().into_iter().next().unwrap();
    let obj_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap().into_iter().next().unwrap();

    let diff = compute_diff(&obj_yaml, &helm_yaml);
    assert!(diff.is_none());
}

#[test]
fn test_compute_diff_simple_diff() {
    let helm_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap().into_iter().next().unwrap();
    let obj_yaml = YamlLoader::load_from_str("a: 1\nb: 3").unwrap().into_iter().next().unwrap();

    let diff = compute_diff(&obj_yaml, &helm_yaml).unwrap();
    let expected_diff = YamlLoader::load_from_str("b: 3").unwrap().into_iter().next().unwrap();

    assert!(deep_equal(&diff, &expected_diff));
}

#[test]
fn test_compute_diff_nested() {
    let helm_yaml = YamlLoader::load_from_str("a:\n  b: 1\n  c: 2").unwrap().into_iter().next().unwrap();
    let obj_yaml = YamlLoader::load_from_str("a:\n  b: 1\n  c: 3").unwrap().into_iter().next().unwrap();

    let diff = compute_diff(&obj_yaml, &helm_yaml).unwrap();
    let expected_diff = YamlLoader::load_from_str("a:\n  c: 3").unwrap().into_iter().next().unwrap();

    assert!(deep_equal(&diff, &expected_diff));
}

#[test]
fn test_compute_diff_array() {
    let helm_yaml = YamlLoader::load_from_str("items:\n  - a\n  - b").unwrap().into_iter().next().unwrap();
    let obj_yaml = YamlLoader::load_from_str("items:\n  - a\n  - c").unwrap().into_iter().next().unwrap();

    let diff = compute_diff(&obj_yaml, &helm_yaml).unwrap();
    let expected_diff = YamlLoader::load_from_str("items:\n  - null\n  - c").unwrap().into_iter().next().unwrap();

    assert!(deep_equal(&diff, &expected_diff));
}

#[test]
fn test_compute_diff_additional_key() {
    let helm_yaml = YamlLoader::load_from_str("a: 1").unwrap().into_iter().next().unwrap();
    let obj_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap().into_iter().next().unwrap();

    let diff = compute_diff(&obj_yaml, &helm_yaml).unwrap();
    let expected_diff = YamlLoader::load_from_str("b: 2").unwrap().into_iter().next().unwrap();

    assert!(deep_equal(&diff, &expected_diff));
}

#[test]
fn test_compute_diff_missing_key() {
    let helm_yaml = YamlLoader::load_from_str("a: 1\nb: 2").unwrap().into_iter().next().unwrap();
    let obj_yaml = YamlLoader::load_from_str("a: 1").unwrap().into_iter().next().unwrap();

    let diff = compute_diff(&obj_yaml, &helm_yaml);
    // Since obj_yaml lacks 'b', and compute_diff only considers keys in obj_yaml,
    // the diff should be None (no differences).
    assert!(diff.is_none());
}

#[test]
fn test_diff_and_common_multiple_identical() {
    let yaml1 = YamlLoader::load_from_str("a: 1\nb: 2").unwrap().into_iter().next().unwrap();
    let yaml2 = YamlLoader::load_from_str("a: 1\nb: 2").unwrap().into_iter().next().unwrap();
    let objs = vec![&yaml1, &yaml2];

    let (base, diffs) = diff_and_common_multiple(&objs, 0.51);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("a: 1\nb: 2").unwrap().into_iter().next().unwrap();
    assert!(deep_equal(base.as_ref().unwrap().as_ref(), &expected_base));

    assert!(diffs.iter().all(|d| d.is_none()));
}

#[test]
fn test_diff_and_common_multiple_different() {
    let yaml1 = YamlLoader::load_from_str("a: 1\nb: 2").unwrap().into_iter().next().unwrap();
    let yaml2 = YamlLoader::load_from_str("a: 1\nb: 3").unwrap().into_iter().next().unwrap();
    let objs = vec![&yaml1, &yaml2];

    let (base, diffs) = diff_and_common_multiple(&objs, 0.51);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("a: 1").unwrap().into_iter().next().unwrap();
    assert!(deep_equal(base.as_ref().unwrap().as_ref(), &expected_base));

    let expected_diffs = [
        YamlLoader::load_from_str("b: 2").unwrap().into_iter().next().unwrap(),
        YamlLoader::load_from_str("b: 3").unwrap().into_iter().next().unwrap(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap().as_ref(), expected_diff));
    }
}

#[test]
fn test_diff_and_common_multiple_three_files_quorum_66() {
    let yaml1 = YamlLoader::load_from_str("a: 1\nb: 2\nc: 3").unwrap().into_iter().next().unwrap();
    let yaml2 = YamlLoader::load_from_str("a: 1\nb: 2\nc: 4").unwrap().into_iter().next().unwrap();
    let yaml3 = YamlLoader::load_from_str("a: 1\nb: 5\nc: 3").unwrap().into_iter().next().unwrap();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let quorum_percentage = 0.66; // 66%
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("a: 1\nb: 2\nc: 3").unwrap().into_iter().next().unwrap();
    assert!(deep_equal(base.as_ref().unwrap().as_ref(), &expected_base));

    let expected_diffs = [
        None, // yaml1 matches the base
        Some(YamlLoader::load_from_str("c: 4").unwrap().into_iter().next().unwrap()),
        Some(YamlLoader::load_from_str("b: 5").unwrap().into_iter().next().unwrap()),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        match expected_diff {
            Some(expected) => {
                assert!(diff.is_some());
                assert!(deep_equal(diff.as_ref().unwrap().as_ref(), expected));
            }
            None => assert!(diff.is_none()),
        }
    }
}

#[test]
fn test_diff_and_common_multiple_shared_empty_hash() {
    // All files share a nested empty hash (e.g. ArgoCD's `syncPolicy: {automated: {}}`).
    // It must survive into the base instead of vanishing from both base and diffs.
    let yaml1 = YamlLoader::load_from_str("syncPolicy:\n  automated: {}\nname: one").unwrap().into_iter().next().unwrap();
    let yaml2 = YamlLoader::load_from_str("syncPolicy:\n  automated: {}\nname: two").unwrap().into_iter().next().unwrap();
    let yaml3 = YamlLoader::load_from_str("syncPolicy:\n  automated: {}\nname: three").unwrap().into_iter().next().unwrap();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let (base, diffs) = diff_and_common_multiple(&objs, 1.0);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("syncPolicy:\n  automated: {}").unwrap().into_iter().next().unwrap();
    assert!(deep_equal(base.as_ref().unwrap().as_ref(), &expected_base));

    let expected_diffs = [
        YamlLoader::load_from_str("name: one").unwrap().into_iter().next().unwrap(),
        YamlLoader::load_from_str("name: two").unwrap().into_iter().next().unwrap(),
        YamlLoader::load_from_str("name: three").unwrap().into_iter().next().unwrap(),
    ];
    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap().as_ref(), expected_diff));
    }
}

#[test]
fn test_diff_and_common_multiple_empty_hash_below_quorum() {
    // One file has `a: {}` while the others have content under `a`. The empty
    // hash misses the quorum, so that file must keep `a: {}` in its diff.
    let yaml1 = YamlLoader::load_from_str("a: {}").unwrap().into_iter().next().unwrap();
    let yaml2 = YamlLoader::load_from_str("a:\n  x: 1").unwrap().into_iter().next().unwrap();
    let yaml3 = YamlLoader::load_from_str("a:\n  x: 1").unwrap().into_iter().next().unwrap();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let (base, diffs) = diff_and_common_multiple(&objs, 1.0);

    // yaml1 contributes Null at key `x`, so nothing can meet the 100% quorum.
    assert!(base.is_none());

    let expected_diffs = [
        YamlLoader::load_from_str("a: {}").unwrap().into_iter().next().unwrap(),
        YamlLoader::load_from_str("a:\n  x: 1").unwrap().into_iter().next().unwrap(),
        YamlLoader::load_from_str("a:\n  x: 1").unwrap().into_iter().next().unwrap(),
    ];
    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap().as_ref(), expected_diff));
    }
}

#[test]
fn test_diff_and_common_multiple_empty_hash_meets_quorum_with_outlier() {
    // Two of three files have `a: {}`; with 51% quorum the empty hash is the
    // base value and the outlier keeps its content in its diff.
    let yaml1 = YamlLoader::load_from_str("a: {}").unwrap().into_iter().next().unwrap();
    let yaml2 = YamlLoader::load_from_str("a: {}").unwrap().into_iter().next().unwrap();
    let yaml3 = YamlLoader::load_from_str("a:\n  x: 1").unwrap().into_iter().next().unwrap();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let (base, diffs) = diff_and_common_multiple(&objs, 0.51);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("a: {}").unwrap().into_iter().next().unwrap();
    assert!(deep_equal(base.as_ref().unwrap().as_ref(), &expected_base));

    assert!(diffs[0].is_none());
    assert!(diffs[1].is_none());
    let expected_diff = YamlLoader::load_from_str("a:\n  x: 1").unwrap().into_iter().next().unwrap();
    assert!(diffs[2].is_some());
    assert!(deep_equal(diffs[2].as_ref().unwrap().as_ref(), &expected_diff));
}

#[test]
fn test_arrays_with_base() {
    let yaml1 = YamlLoader::load_from_str(
        "app-backend:\n  migrationJob:\n    extraEnv:\n      - name: POSTGRESQL_CONNECTION_STRING\n        valueFrom:\n          secretKeyRef:\n            name: app-backend\n            key: POSTGRESQL_CONNECTION_STRING",
    )
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let yaml2 = YamlLoader::load_from_str(
        "app-backend:\n  migrationJob:\n    extraEnv:\n      - name: POSTGRESQL_CONNECTION_STRING\n        valueFrom:\n          secretKeyRef:\n            name: app-backend\n            key: POSTGRESQL_CONNECTION_STRING",
    )
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let yaml3 = YamlLoader::load_from_str(
        "app-backend:\n  migrationJob:\n    extraEnv:\n      - name: POSTGRESQL_CONNECTION_STRING\n        valueFrom:\n          secretKeyRef:\n            name: app-backend\n            key: POSTGRESQL_CONNECTION_STRING1",
    )
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let objs = vec![&yaml1, &yaml2, &yaml3];
    let quorum_percentage = 0.51;
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str(
        "app-backend:\n  migrationJob:\n    extraEnv:\n      - name: POSTGRESQL_CONNECTION_STRING\n        valueFrom:\n          secretKeyRef:\n            name: app-backend\n            key: POSTGRESQL_CONNECTION_STRING",
    )
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert!(deep_equal(base.as_ref().unwrap().as_ref(), &expected_base));

    let expected_diffs = [
        None, // yaml1 matches the base
        None, // yaml2 matches the base
        Some(
            YamlLoader::load_from_str(
                "app-backend:\n  migrationJob:\n    extraEnv:\n      - name: POSTGRESQL_CONNECTION_STRING\n        valueFrom:\n          secretKeyRef:\n            name: app-backend\n            key: POSTGRESQL_CONNECTION_STRING1",
            )
                .unwrap()
                .into_iter()
                .next()
                .unwrap(),
        ),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        match expected_diff {
            Some(expected) => {
                assert!(diff.is_some());
                assert!(deep_equal(diff.as_ref().unwrap().as_ref(), expected));
            }
            None => assert!(diff.is_none()),
        }
    }
}