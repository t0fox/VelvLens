use std::collections::BTreeMap;

use crate::model::ProxyConfig;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DuplicateReport {
    pub exact_groups: Vec<Vec<usize>>,
    pub semantic_groups: Vec<Vec<usize>>,
    pub exact_duplicates: usize,
    pub semantic_duplicates: usize,
}

pub fn deduplicate(configs: &mut [ProxyConfig]) -> DuplicateReport {
    let mut exact: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut semantic: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, config) in configs.iter().enumerate() {
        exact.entry(config.raw_uri.clone()).or_default().push(index);
        semantic
            .entry(config.semantic_key())
            .or_default()
            .push(index);
    }

    let exact_groups: Vec<Vec<usize>> = exact
        .into_values()
        .filter(|group| group.len() > 1)
        .collect();
    let semantic_groups: Vec<Vec<usize>> = semantic
        .into_values()
        .filter(|group| group.len() > 1)
        .collect();
    for group in &exact_groups {
        for &index in group {
            configs[index].metadata.exact_duplicate_count = group.len();
        }
    }
    for (group_index, group) in semantic_groups.iter().enumerate() {
        for &index in group {
            configs[index].metadata.semantic_duplicate_group = Some(group_index);
        }
    }
    DuplicateReport {
        exact_duplicates: exact_groups
            .iter()
            .map(Vec::len)
            .sum::<usize>()
            .saturating_sub(exact_groups.len()),
        semantic_duplicates: semantic_groups
            .iter()
            .map(Vec::len)
            .sum::<usize>()
            .saturating_sub(semantic_groups.len()),
        exact_groups,
        semantic_groups,
    }
}
