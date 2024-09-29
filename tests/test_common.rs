use yabe::diff::{compute_diff, diff_and_common_multiple};
use yabe::deep_equal::deep_equal;
use yaml_rust2::{YamlLoader, Yaml};

#[test]
fn test_empty_documents() {
    let yaml1 = YamlLoader::load_from_str("").unwrap_or_default().into_iter().next().unwrap_or(Yaml::Null);
    let yaml2 = YamlLoader::load_from_str("").unwrap_or_default().into_iter().next().unwrap_or(Yaml::Null);

    assert!(deep_equal(&yaml1, &yaml2));

    let diff = compute_diff(&yaml1, &yaml2);
    assert!(diff.is_none());

    let objs = vec![&yaml1, &yaml2];
    let (base, diffs) = diff_and_common_multiple(&objs, 0.51);

    assert!(base.is_some());
    assert!(deep_equal(base.as_ref().unwrap().as_ref(), &Yaml::Null));
    assert!(diffs.iter().all(|d| d.is_none()));
}

#[test]
fn test_different_types_same_key() {
    let yaml1 = YamlLoader::load_from_str("key: value").unwrap().into_iter().next().unwrap();
    let yaml2 = YamlLoader::load_from_str("key:\n  subkey: value").unwrap().into_iter().next().unwrap();

    let objs = vec![&yaml1, &yaml2];
    let (base, diffs) = diff_and_common_multiple(&objs, 0.51);

    assert!(base.is_none());

    let expected_diffs = [
        YamlLoader::load_from_str("key: value").unwrap().into_iter().next().unwrap(),
        YamlLoader::load_from_str("key:\n  subkey: value").unwrap().into_iter().next().unwrap(),
    ];

    for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap().as_ref(), expected_diff));
    }
}

// The test for null values is commented out because it's pending further implementation.
// Uncomment and adjust if necessary.

// #[test]
// fn test_null_values() {
//     let yaml1 = YamlLoader::load_from_str("key: null").unwrap().into_iter().next().unwrap();
//     let yaml2 = YamlLoader::load_from_str("key: null").unwrap().into_iter().next().unwrap();
//     let yaml3 = YamlLoader::load_from_str("key: value").unwrap().into_iter().next().unwrap();
//     let objs = vec![&yaml1, &yaml2, &yaml3];

//     let quorum_percentage = 0.66; // 66%
//     let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);
//     assert!(base.is_none());

//     // Expected diffs
//     let expected_diffs = [
//         None, // yaml1 matches the base
//         None, // yaml2 matches the base
//         Some(YamlLoader::load_from_str("key: value").unwrap().into_iter().next().unwrap()),
//     ];

//     for (diff, expected_diff) in diffs.iter().zip(expected_diffs.iter()) {
//         match expected_diff {
//             Some(expected) => {
//                 assert!(diff.is_some());
//                 assert!(deep_equal(diff.as_ref().unwrap().as_ref(), expected));
//             }
//             None => assert!(diff.is_none()),
//         }
//     }
// }

#[test]
fn test_quorum_base_determination() {
    let yaml1 = YamlLoader::load_from_str("key: value1").unwrap().into_iter().next().unwrap();
    let yaml2 = YamlLoader::load_from_str("key: value1").unwrap().into_iter().next().unwrap();
    let yaml3 = YamlLoader::load_from_str("key: value2").unwrap().into_iter().next().unwrap();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let quorum_percentage = 0.66; // 66%
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    assert!(base.is_some());
    let expected_base = YamlLoader::load_from_str("key: value1").unwrap().into_iter().next().unwrap();
    assert!(deep_equal(base.as_ref().unwrap().as_ref(), &expected_base));

    // diffs should contain deviations
    assert!(diffs[2].is_some());
    let expected_diff = YamlLoader::load_from_str("key: value2").unwrap().into_iter().next().unwrap();
    assert!(deep_equal(diffs[2].as_ref().unwrap().as_ref(), &expected_diff));
}

#[test]
fn test_recursive_diff_with_nested_structures() {
    let yaml1 = YamlLoader::load_from_str("a:\n  b:\n    c: 1\n    f: 2").unwrap().into_iter().next().unwrap();
    let yaml2 = YamlLoader::load_from_str("a:\n  b:\n    c: 1\n    e: 2").unwrap().into_iter().next().unwrap();
    let yaml3 = YamlLoader::load_from_str("a:\n  b:\n    c: 1\n    d: 2").unwrap().into_iter().next().unwrap();
    let objs = vec![&yaml1, &yaml2, &yaml3];

    let quorum_percentage = 0.51;
    let (base, diffs) = diff_and_common_multiple(&objs, quorum_percentage);

    // Expected base
    let expected_base_str = "a:\n  b:\n    c: 1";
    let expected_base = YamlLoader::load_from_str(expected_base_str).unwrap().into_iter().next().unwrap();
    assert!(base.is_some());
    assert!(deep_equal(base.as_ref().unwrap().as_ref(), &expected_base));

    // Expected diffs
    let expected_diffs_strs = [
        "a:\n  b:\n    f: 2",
        "a:\n  b:\n    e: 2",
        "a:\n  b:\n    d: 2",
    ];
    for (diff, expected_diff_str) in diffs.iter().zip(expected_diffs_strs.iter()) {
        let expected_diff = YamlLoader::load_from_str(expected_diff_str).unwrap().into_iter().next().unwrap();
        assert!(diff.is_some());
        assert!(deep_equal(diff.as_ref().unwrap().as_ref(), &expected_diff));
    }
}