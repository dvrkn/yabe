use std::borrow::Cow;
use yaml_rust2::Yaml;

/// Merges two YAML documents.
pub fn merge_yaml<'a>(base: &'a Yaml, override_yaml: &'a Yaml) -> Cow<'a, Yaml> {
    match (base, override_yaml) {
        (Yaml::Hash(base_hash), Yaml::Hash(override_hash)) => {
            let mut merged = base_hash.clone();
            for (key, override_value) in override_hash {
                merged.entry(key.clone())
                    .and_modify(|base_value| {
                        let merged_value = merge_yaml(base_value, override_value);
                        *base_value = merged_value.into_owned();
                    })
                    .or_insert_with(|| override_value.clone());
            }
            Cow::Owned(Yaml::Hash(merged))
        }
        (_, override_val) => Cow::Borrowed(override_val),
    }
}