pub fn sort(items: Vec<String>) -> Vec<String> {
    let mut items = items;
    let n = items.len();

    for i in (0..n / 2).rev() {
        heapify(&mut items, n, i);
    }

    for i in (0..n).rev() {
        items.swap(0, i);
        heapify(&mut items, i, 0);
    }

    items
}

fn heapify(items: &mut Vec<String>, n: usize, i: usize) {
    let mut largest = i;
    let left = 2 * i + 1;
    let right = 2 * i + 2;

    if left < n && items[left] > items[largest] {
        largest = left;
    }

    if right < n && items[right] > items[largest] {
        largest = right;
    }

    if largest != i {
        items.swap(i, largest);
        heapify(items, n, largest);
    }
}
