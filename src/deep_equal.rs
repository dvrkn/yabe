use yaml_rust2::Yaml;

/// Recursively checks if two Yaml values are deeply equal.
pub fn deep_equal(a: &Yaml, b: &Yaml) -> bool {
    match (a, b) {
        (Yaml::Real(a_str), Yaml::Real(b_str)) => a_str == b_str,
        (Yaml::Integer(a_int), Yaml::Integer(b_int)) => a_int == b_int,
        (Yaml::String(a_str), Yaml::String(b_str)) => a_str == b_str,
        (Yaml::Boolean(a_bool), Yaml::Boolean(b_bool)) => a_bool == b_bool,
        (Yaml::Array(a_vec), Yaml::Array(b_vec)) => {
            if a_vec.len() != b_vec.len() {
                false
            } else {
                for (a_item, b_item) in a_vec.iter().zip(b_vec.iter()) {
                    if !deep_equal(a_item, b_item) {
                        return false;
                    }
                }
                true
            }
        }
        (Yaml::Hash(a_hash), Yaml::Hash(b_hash)) => {
            if a_hash.len() != b_hash.len() {
                false
            } else {
                for (a_key, a_value) in a_hash.iter() {
                    if let Some(b_value) = b_hash.get(a_key) {
                        if !deep_equal(a_value, b_value) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
        }
        (Yaml::Null, Yaml::Null) => true,
        _ => false,
    }
}