# Running this cut tool:

## Step 1:

```
cargo run -- -f2 sample.tsv
```

## Step 2:

```
cargo run -- -f1 -d, fourchords.csv | head -n5
```

&

```
cargo run -- -f1 sample.tsv
```

## Step 3:

```
cargo run -- -f1,2 sample.tsv
```

&

```
cargo run -- -d,  -f"1 2" fourchords.csv | head -n5
```

## Step 4:

```
tail -n5 fourchords.csv |cargo run -- -d,  -f"1 2"
```
