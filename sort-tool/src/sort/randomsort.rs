use std::collections::HashMap;

use itertools::Itertools;
use rand::{rngs::OsRng, Rng};

pub fn sort(items: Vec<String>) -> Vec<String> {
    let mut rng = OsRng;

    let item_to_random_value = items
        .iter()
        .map(|item| (item.clone(), rng.gen::<u64>()))
        .collect::<HashMap<_, _>>();

    items
        .into_iter()
        .sorted_by_key(|item| item_to_random_value.get(item).copied().unwrap_or(u64::MAX))
        .collect_vec()
}
