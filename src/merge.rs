use std::borrow::Cow;
use yaml_rust2::Yaml;

/// Merges two YAML documents.
pub fn merge_yaml<'a>(base: &'a Yaml, override_yaml: &'a Yaml) -> Cow<'a, Yaml> {
    match (base, override_yaml) {
        (Yaml::Hash(_), Yaml::Hash(_)) => {
            // Clone the base once and merge overrides in place; cloning per
            // recursion level would make allocation scale with tree depth.
            let mut merged = base.clone();
            merge_in_place(&mut merged, override_yaml);
            Cow::Owned(merged)
        }
        (_, override_val) => Cow::Borrowed(override_val),
    }
}

fn merge_in_place(base: &mut Yaml, override_yaml: &Yaml) {
    match (base, override_yaml) {
        (Yaml::Hash(base_hash), Yaml::Hash(override_hash)) => {
            for (key, override_value) in override_hash {
                if let Some(base_value) = base_hash.get_mut(key) {
                    merge_in_place(base_value, override_value);
                } else {
                    base_hash.insert(key.clone(), override_value.clone());
                }
            }
        }
        (base_slot, override_val) => *base_slot = override_val.clone(),
    }
}