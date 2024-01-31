## Running the sort tool

```
cargo run words.txt | uniq | head -n5
```

With uniqueness inbuilt

```
cargo run -- -u words.txt | head -n5
```
