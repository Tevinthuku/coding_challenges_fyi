use std::iter;

pub fn sort(items: Vec<String>) -> Vec<String> {
    if items.len() < 2 {
        return items;
    }

    let pivot_index = items.len() - 1;
    let pivot = items[pivot_index].clone();

    let mut left = vec![];
    let mut right = vec![];

    let slice = &items[..pivot_index];

    for item in slice {
        if item < &pivot {
            left.push(item.clone());
        } else {
            right.push(item.clone());
        }
    }

    sort(left)
        .into_iter()
        .chain(iter::once(pivot))
        .chain(sort(right))
        .collect()
}
