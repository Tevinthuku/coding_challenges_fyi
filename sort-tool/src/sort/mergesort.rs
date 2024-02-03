use std::collections::VecDeque;

pub fn sort(items: Vec<String>) -> Vec<String> {
    if items.len() < 2 {
        return items;
    }

    let middle = items.len() / 2;
    let left = items[..middle].to_vec();
    let right = items[middle..].to_vec();

    merge(sort(left), sort(right))
}

fn merge(left: Vec<String>, right: Vec<String>) -> Vec<String> {
    let mut left = VecDeque::from(left);
    let mut right = VecDeque::from(right);
    let mut sorted = vec![];

    loop {
        match (left.is_empty(), right.is_empty()) {
            (false, false) => {
                if left[0] <= right[0] {
                    // its safe to unwrap here because we know the VecDeque is not empty
                    sorted.push(left.pop_front().unwrap());
                } else {
                    // its safe to unwrap here because we know the VecDeque is not empty
                    sorted.push(right.pop_front().unwrap());
                }
            }
            _ => {
                sorted.extend(left);
                sorted.extend(right);
                break;
            }
        }
    }

    sorted
}
