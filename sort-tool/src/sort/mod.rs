use itertools::Itertools;

mod bubblesort;
mod heapsort;
mod mergesort;
mod quicksort;
mod randomsort;

pub fn sort(items: Vec<String>, algorithm: &str) -> Vec<String> {
    match algorithm {
        "quicksort" => quicksort::sort(items),
        "mergesort" => mergesort::sort(items),
        "bubblesort" => bubblesort::sort(items),
        "heapsort" => heapsort::sort(items),
        "randomsort" => randomsort::sort(items),
        _ => items.into_iter().sorted().collect_vec(),
    }
}
