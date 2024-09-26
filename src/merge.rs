use yaml_rust2::{Yaml};


// Add this function to handle YAML merging
pub fn merge_yaml(base: &Yaml, override_yaml: &Yaml) -> Yaml {
    match (base, override_yaml) {
        (Yaml::Hash(base_hash), Yaml::Hash(override_hash)) => {
            let mut merged = base_hash.clone();
            for (key, override_value) in override_hash.iter() {
                if let Some(base_value) = merged.get_mut(key) {
                    let merged_value = merge_yaml(base_value, override_value);
                    merged.insert(key.clone(), merged_value);
                } else {
                    merged.insert(key.clone(), override_value.clone());
                }
            }
            Yaml::Hash(merged)
        }
        // For arrays and other types, override the base value
        (_, override_val) => override_val.clone(),
    }
}