pub fn sort(items: Vec<String>) -> Vec<String> {
    let mut items = items;
    let mut swapped = true;

    while swapped {
        swapped = false;

        for i in 1..items.len() {
            if items[i - 1] > items[i] {
                items.swap(i - 1, i);
                swapped = true;
            }
        }
    }

    items
}
