use yaml_rust2::yaml::{Array, Hash, Yaml};
use std::borrow::Cow;

pub fn sort_yaml<'a>(doc: Cow<'a, Yaml>, config: &Yaml) -> Cow<'a, Yaml> {
    match &*doc {
        Yaml::Array(v) => {
            if let Some(sort_key) = config["sortKey"].as_str() {
                let mut new_v = v.clone();
                array_sorter(&mut new_v, sort_key);
                for x in &mut new_v {
                    *x = sort_yaml(Cow::Borrowed(x), config).into_owned();
                }
                Cow::Owned(Yaml::Array(new_v))
            } else {
                doc
            }
        }
        Yaml::Hash(h) => {
            if let Some(pre_order_vec) = config["preOrder"].as_vec() {
                let mut new_h = h.clone();
                let pre_order = pre_order_vec
                    .iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<&str>>();
                hash_sorter(&mut new_h, &pre_order);
                for (_, v) in &mut new_h {
                    *v = sort_yaml(Cow::Borrowed(v), config).into_owned();
                }
                Cow::Owned(Yaml::Hash(new_h))
            } else {
                doc
            }
        }
        _ => doc,
    }
}

pub fn hash_sorter(hash: &mut Hash, pre_order: &[&str]) {
    let mut result = Hash::new();

    // Sort the hash by the pre_order array
    for key in pre_order {
        if let Some((k, v)) = hash.remove_entry(&Yaml::String((*key).to_string())) {
            result.insert(k, v);
        }
    }

    // Collect the remaining keys
    let mut hash_keys: Vec<Yaml> = hash.keys().cloned().collect();
    hash_keys.sort_by(|a, b| a.cmp(b));

    for key in hash_keys {
        if let Some((k, v)) = hash.remove_entry(&key) {
            result.insert(k, v);
        }
    }

    *hash = result;
}

pub fn array_sorter(array: &mut Array, sort_key: &str) {
    array.sort_by(|a, b| match (a[sort_key].as_str(), b[sort_key].as_str()) {
        (Some(a_str), Some(b_str)) => a_str.cmp(b_str),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
}