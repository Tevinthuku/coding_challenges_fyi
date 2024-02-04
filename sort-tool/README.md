## Running the sort tool

```
cargo run words.txt | uniq | head -n5
```

With uniqueness inbuilt

```
cargo run -- -u words.txt | head -n5
```

With an algorithm

```
cargo run -- -u -sort=heapsort words.txt | head -n5

cargo run -- -u -sort=bubblesort words.txt | head -n5

cargo run -- -u -sort=quicksort words.txt | head -n5

cargo run -- -u -sort=mergesort words.txt | head -n5
```

Random sort

```
cargo run --  -sort=randomsort words.txt | head -n5

cargo run --  -R words.txt | head -n5

cargo run --  -random-sort words.txt | head -n5
```
