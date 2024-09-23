use std::fs;
use std::collections::HashSet;
use std::error::Error;
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};
use yaml_rust2::yaml::Hash;

/// Recursively checks if two Yaml values are deeply equal.
fn deep_equal(a: &Yaml, b: &Yaml) -> bool {
    match (a, b) {
        // Compare numbers, strings, booleans directly
        (Yaml::Real(a_str), Yaml::Real(b_str)) => a_str == b_str,
        (Yaml::Integer(a_int), Yaml::Integer(b_int)) => a_int == b_int,
        (Yaml::String(a_str), Yaml::String(b_str)) => a_str == b_str,
        (Yaml::Boolean(a_bool), Yaml::Boolean(b_bool)) => a_bool == b_bool,

        // Compare arrays element-wise
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
        },

        // Compare hashes (maps) by keys and values
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
        },

        // Both are null
        (Yaml::Null, Yaml::Null) => true,

        // Types differ
        _ => false,
    }
}

/// Recursively computes the common base and differences among multiple Yaml objects.
fn diff_and_common_multiple(objs: &[Yaml]) -> (Option<Yaml>, Vec<Option<Yaml>>) {
    if objs.is_empty() {
        return (None, vec![]);
    }

    // Check if all objects are deeply equal
    if objs.iter().all(|val| deep_equal(val, &objs[0])) {
        // All objects are identical; set base and no diffs
        return (Some(objs[0].clone()), vec![None; objs.len()]);
    }

    // Collect types of each object
    let types: Vec<&str> = objs.iter().map(|obj| match obj {
        Yaml::Null => "null",
        Yaml::Boolean(_) => "bool",
        Yaml::Integer(_) => "int",
        Yaml::Real(_) => "real",
        Yaml::String(_) => "string",
        Yaml::Array(_) => "array",
        Yaml::Hash(_) => "hash",
        _ => "unknown",
    }).collect();

    let type_set: HashSet<&str> = types.iter().cloned().collect();

    // If types differ or any is null, treat entire values as diffs
    if type_set.len() > 1 || types.contains(&"null") {
        return (None, objs.iter().map(|obj| Some(obj.clone())).collect());
    }

    let obj_type = types[0];

    // Handle primitive types (non-object, non-array)
    if obj_type != "hash" && obj_type != "array" {
        // Values differ; include them in diffs
        return (None, objs.iter().map(|obj| Some(obj.clone())).collect());
    }

    // Handle arrays
    if obj_type == "array" {
        // Compare arrays as whole units
        if objs.iter().all(|val| deep_equal(val, &objs[0])) {
            // Arrays are identical; set as base
            return (Some(objs[0].clone()), vec![None; objs.len()]);
        } else {
            // Arrays differ; include them in diffs
            return (None, objs.iter().map(|obj| Some(obj.clone())).collect());
        }
    }

    // Handle hashes (maps)
    if obj_type == "hash" {
        // Collect all unique keys (assuming keys are strings)
        let mut all_keys = HashSet::new();
        for obj in objs {
            if let Yaml::Hash(ref h) = obj {
                for key in h.keys() {
                    if let Yaml::String(ref key_str) = key {
                        all_keys.insert(key_str.clone());
                    } else {
                        // Non-string key encountered; handle as needed
                        println!("Non-string key encountered in hash");
                    }
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
            let key_yaml = Yaml::String(key.clone());

            // Collect values at current key from all objects
            let values_at_key: Vec<Yaml> = objs.iter().map(|obj| {
                if let Yaml::Hash(ref h) = obj {
                    h.get(&key_yaml).cloned().unwrap_or(Yaml::Null)
                } else {
                    Yaml::Null
                }
            }).collect();

            // Recursively compute base and diffs for the current key
            let (sub_base, sub_diffs) = diff_and_common_multiple(&values_at_key);

            // Add to base if common
            if let Some(sub_base_val) = sub_base {
                base_hash.insert(key_yaml.clone(), sub_base_val);
                has_base = true;
            }

            // Add to diffs if different
            for (i, sub_diff) in sub_diffs.into_iter().enumerate() {
                if let Some(sub_diff_val) = sub_diff {
                    diffs[i].insert(key_yaml.clone(), sub_diff_val);
                    has_diffs[i] = true;
                }
            }
        }

        // Prepare base and diffs for return
        let base = if has_base { Some(Yaml::Hash(base_hash)) } else { None };
        let diffs_result: Vec<Option<Yaml>> = diffs.into_iter().enumerate().map(|(i, h)| {
            if has_diffs[i] {
                Some(Yaml::Hash(h))
            } else {
                None
            }
        }).collect();

        return (base, diffs_result);
    }

    // Should not reach here; treat as diffs
    return (None, objs.iter().map(|obj| Some(obj.clone())).collect());
}

fn main() -> Result<(), Box<dyn Error>> {
    // List of input YAML filenames
    let input_filenames = vec!["file1.yaml", "file2.yaml", "file3.yaml"];

    // Read and parse each YAML file into an object
    let mut objs = Vec::new();
    for filename in &input_filenames {
        // Read file content
        let content = fs::read_to_string(filename)?;
        // Parse YAML documents
        let docs = YamlLoader::load_from_str(&content)?;
        if docs.is_empty() {
            panic!("No YAML documents in {}", filename);
        }
        // Assume the first document in the file
        objs.push(docs[0].clone());
    }

    // Compute the common base and differences among the objects
    let (base, diffs) = diff_and_common_multiple(&objs);

    // Write the base YAML file if it exists
    if let Some(base_yaml) = base {
        let mut out_str = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(&base_yaml)?;
        }
        fs::write("base.yaml", out_str)?;
        println!("Base YAML written to base.yaml");
    }

    // Write diff files for each input file
    for (i, diff) in diffs.iter().enumerate() {
        if let Some(diff_yaml) = diff {
            let mut out_str = String::new();
            {
                let mut emitter = YamlEmitter::new(&mut out_str);
                emitter.dump(diff_yaml)?;
            }
            let diff_filename = format!("diff{}.yaml", i + 1);
            fs::write(&diff_filename, out_str)?;
            println!("Difference for {} written to {}", input_filenames[i], diff_filename);
        }
    }

    Ok(())
}