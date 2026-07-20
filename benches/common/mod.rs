use yaml_rust2::yaml::{Hash, Yaml};

/// Builds a synthetic GitOps-values-shaped document: a hash tree of the given
/// depth and width whose leaves are mostly identical across variants, with one
/// variant-dependent leaf per hash node (~1/width divergence). Leaf mix covers
/// strings, ints, arrays, and empty hashes (the `syncPolicy: {automated: {}}`
/// shape).
pub fn gen_doc(depth: usize, width: usize, variant: i64) -> Yaml {
    build_hash(depth, width, variant, 1)
}

fn build_hash(depth: usize, width: usize, variant: i64, salt: i64) -> Yaml {
    let mut h = Hash::new();
    for i in 0..width {
        let key = Yaml::String(format!("key_{}_{}", depth, i));
        let val = if depth == 0 {
            match i {
                0 => Yaml::String(format!("variant-{}-{}", variant, salt)),
                1 => Yaml::Integer(salt * 31 + i as i64),
                2 => Yaml::Array(vec![
                    Yaml::String(format!("item-{}", salt)),
                    Yaml::Boolean(true),
                    Yaml::Integer(i as i64),
                ]),
                3 => Yaml::Hash(Hash::new()),
                _ => Yaml::String(format!("common-{}-{}", salt, i)),
            }
        } else {
            build_hash(depth - 1, width, variant, salt.wrapping_mul(7).wrapping_add(i as i64))
        };
        h.insert(key, val);
    }
    Yaml::Hash(h)
}

/// Total node count (hashes, arrays, and scalars) of a document.
#[allow(dead_code)]
pub fn count_nodes(doc: &Yaml) -> usize {
    match doc {
        Yaml::Hash(h) => 1 + h.iter().map(|(k, v)| count_nodes(k) + count_nodes(v)).sum::<usize>(),
        Yaml::Array(a) => 1 + a.iter().map(count_nodes).sum::<usize>(),
        _ => 1,
    }
}

/// Sort configuration matching the generated key names.
pub fn sort_config() -> Yaml {
    let src = "sortKey: name\npreOrder:\n  - key_0_3\n  - key_0_1\n  - key_0_0";
    yaml_rust2::YamlLoader::load_from_str(src)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}
