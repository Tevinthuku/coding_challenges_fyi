# Running the commands

## Reading file bytes

```
cargo run -- -c test.txt
```

## Reading number of lines

```
cargo run -- -l test.txt
```

## Count of words

```
cargo run -- -w test.txt
```

## Count number of characters

```
cargo run -- -m test.txt
```

## Read from command line

```
cat test.txt | cargo run -- -l
```
