use yaml_rust2::yaml::{Array, Hash, Yaml};
use std::borrow::Cow;

pub fn sort_yaml<'a>(doc: &'a Yaml, config: &Yaml) -> Cow<'a, Yaml> {
    let sort_key = config["sortKey"].as_str();
    let pre_order: Option<Vec<&str>> = config["preOrder"]
        .as_vec()
        .map(|v| v.iter().filter_map(|x| x.as_str()).collect());

    // Clone the document once and sort in place; cloning per recursion level
    // would make allocation scale with tree depth.
    match doc {
        Yaml::Array(_) if sort_key.is_some() => {
            let mut owned = doc.clone();
            sort_in_place(&mut owned, sort_key, pre_order.as_deref());
            Cow::Owned(owned)
        }
        Yaml::Hash(_) if pre_order.is_some() => {
            let mut owned = doc.clone();
            sort_in_place(&mut owned, sort_key, pre_order.as_deref());
            Cow::Owned(owned)
        }
        _ => Cow::Borrowed(doc),
    }
}

fn sort_in_place(doc: &mut Yaml, sort_key: Option<&str>, pre_order: Option<&[&str]>) {
    match doc {
        Yaml::Array(v) => {
            if let Some(key) = sort_key {
                array_sorter(v, key);
                for x in v {
                    sort_in_place(x, sort_key, pre_order);
                }
            }
        }
        Yaml::Hash(h) => {
            if let Some(order) = pre_order {
                hash_sorter(h, order);
                for (_, v) in h {
                    sort_in_place(v, sort_key, pre_order);
                }
            }
        }
        _ => {}
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