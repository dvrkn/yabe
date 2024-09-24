use std::borrow::Cow;
use std::collections::HashSet;

use log::debug;
use yaml_rust2::yaml::{Hash, Yaml};
use crate::deep_equal::deep_equal;

/// Recursively computes the difference between an override YAML object and the helm values YAML object.
pub fn compute_diff(obj: &Yaml, helm: &Yaml) -> Option<Yaml> {
    if deep_equal(obj, helm) {
        None
    } else {
        match (obj, helm) {
            (Yaml::Hash(obj_hash), Yaml::Hash(helm_hash)) => {
                let mut diff_hash = Hash::new();
                for (key, obj_value) in obj_hash {
                    let helm_value = helm_hash.get(key).unwrap_or(&Yaml::Null);
                    if let Some(diff_value) = compute_diff(obj_value, helm_value) {
                        diff_hash.insert(key.clone(), diff_value);
                    }
                }
                if diff_hash.is_empty() {
                    None
                } else {
                    Some(Yaml::Hash(diff_hash))
                }
            }
            (Yaml::Array(obj_array), Yaml::Array(helm_array)) => {
                if obj_array.len() != helm_array.len() {
                    Some(obj.clone())
                } else {
                    let mut diffs = Vec::new();
                    let mut has_diff = false;
                    for (obj_item, helm_item) in obj_array.iter().zip(helm_array.iter()) {
                        if let Some(diff_item) = compute_diff(obj_item, helm_item) {
                            diffs.push(diff_item);
                            has_diff = true;
                        } else {
                            diffs.push(Yaml::Null);
                        }
                    }
                    if has_diff {
                        Some(Yaml::Array(diffs))
                    } else {
                        None
                    }
                }
            }
            _ => Some(obj.clone()),
        }
    }
}

/// Recursively computes the common base and differences among multiple Yaml objects.
pub fn diff_and_common_multiple<'a>(
    objs: &'a [&'a Yaml],
    _helm_values: Option<&'a Yaml>, // Not used in this function
) -> (Option<Cow<'a, Yaml>>, Vec<Option<Cow<'a, Yaml>>>) {
    debug!("diff_and_common_multiple called with {} objects.", objs.len());

    if objs.is_empty() {
        debug!("No objects to process. Returning.");
        return (None, vec![]);
    }

    // Check if all objects are deeply equal
    if objs.iter().all(|val| deep_equal(val, objs[0])) {
        debug!("All objects are deeply equal. Including in base.");
        return (Some(Cow::Borrowed(objs[0])), vec![None; objs.len()]);
    }

    // Collect types of each object
    let types: Vec<&str> = objs
        .iter()
        .map(|obj| match obj {
            Yaml::Null => "null",
            Yaml::Boolean(_) => "bool",
            Yaml::Integer(_) => "int",
            Yaml::Real(_) => "real",
            Yaml::String(_) => "string",
            Yaml::Array(_) => "array",
            Yaml::Hash(_) => "hash",
            _ => "unknown",
        })
        .collect();

    let type_set: HashSet<&str> = types.iter().cloned().collect();

    // If types differ or any is null, include them in diffs
    if type_set.len() > 1 || types.contains(&"null") {
        debug!("Types differ or contain null. Including entire values in diffs.");
        return (
            None,
            objs.iter().map(|obj| Some(Cow::Borrowed(*obj))).collect(),
        );
    }

    let obj_type = types[0];

    // Handle primitive types (non-object, non-array)
    if obj_type != "hash" && obj_type != "array" {
        debug!("Handling primitive types.");
        return (
            None,
            objs.iter().map(|obj| Some(Cow::Borrowed(*obj))).collect(),
        );
    }

    // Handle arrays
    if obj_type == "array" {
        debug!("Handling arrays.");
        // Compare arrays as whole units
        if objs.iter().all(|val| deep_equal(val, objs[0])) {
            debug!("Arrays are identical. Including in base.");
            return (Some(Cow::Borrowed(objs[0])), vec![None; objs.len()]);
        } else {
            debug!("Arrays differ. Including in diffs.");
            return (
                None,
                objs.iter().map(|obj| Some(Cow::Borrowed(*obj))).collect(),
            );
        }
    }

    // Handle hashes (maps)
    if obj_type == "hash" {
        debug!("Handling hashes (maps).");
        debug!("Collecting all unique keys.");
        // Collect all unique keys
        let mut all_keys = HashSet::new();
        for obj in objs {
            if let Yaml::Hash(ref h) = obj {
                for key in h.keys() {
                    all_keys.insert(key);
                }
            }
        }

        // Initialize base hash and diffs
        let mut base_hash = Hash::new();
        let mut diffs: Vec<Hash> = vec![Hash::new(); objs.len()];
        let mut has_base = false;
        let mut has_diffs = vec![false; objs.len()];

        // Iterate over all keys
        for key in &all_keys {
            debug!("Processing key: {:?}", key);

            // Collect values at current key from all objects
            let values_at_key: Vec<&Yaml> = objs
                .iter()
                .map(|obj| {
                    if let Yaml::Hash(ref h) = obj {
                        h.get(*key).unwrap_or(&Yaml::Null)
                    } else {
                        &Yaml::Null
                    }
                })
                .collect();

            debug!("Values at key: {:?}", values_at_key);

            // Recursively compute base and diffs for the current key
            debug!("Recursively computing diff for key: {:?}", key);
            let (sub_base, sub_diffs) = diff_and_common_multiple(&values_at_key, None);

            // Add to base if common
            if let Some(sub_base_val) = sub_base {
                debug!("Base value at key {:?}: {:?}", key, sub_base_val);
                base_hash.insert((*key).clone(), sub_base_val.into_owned());
                has_base = true;
            }

            // Add to diffs if different
            for (i, sub_diff) in sub_diffs.into_iter().enumerate() {
                if let Some(sub_diff_val) = sub_diff {
                    debug!("Diff for object {} at key {:?}: {:?}", i, key, sub_diff_val);
                    diffs[i].insert((*key).clone(), sub_diff_val.into_owned());
                    has_diffs[i] = true;
                }
            }
        }

        // Prepare base and diffs for return
        let base = if has_base {
            debug!("Base hash constructed.");
            Some(Cow::Owned(Yaml::Hash(base_hash)))
        } else {
            debug!("No base hash constructed.");
            None
        };

        let diffs_result: Vec<Option<Cow<'a, Yaml>>> = diffs
            .into_iter()
            .enumerate()
            .map(|(i, h)| {
                if has_diffs[i] {
                    Some(Cow::Owned(Yaml::Hash(h)))
                } else {
                    None
                }
            })
            .collect();

        return (base, diffs_result);
    }

    // Should not reach here; treat as diffs
    debug!("Unhandled object type. Including entire values in diffs.");
    (
        None,
        objs.iter().map(|obj| Some(Cow::Borrowed(*obj))).collect(),
    )
}