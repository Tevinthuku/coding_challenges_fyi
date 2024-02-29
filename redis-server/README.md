# Challenge Link:

https://codingchallenges.fyi/challenges/challenge-redis

## How to run

### Running the server

### Build (at the root)

```
cargo build --release --bin redis-server
```

### Run the server

```
./target/release/redis-server
```

### Passing commands from the cli-client

```
redis-cli set Name Tev
```

```
redis-cli get Name
```

## Commands handled

Implementation details can be seen at the `cmd` directory:

- decr
- del
- echo
- exists
- get
- incr
- lpush
- ping
- rpush
- save
- set
  - Expiry flags: "ex" | "px" | "exat" | "pxat"
  - get flag: -> Returns existing value

## My benchmark results after building with the release flag

```
redis-benchmark -t set,get, -n 100000 -q
SET: 139470.02 requests per second, p50=0.191 msec
GET: 148367.95 requests per second, p50=0.183 msec
```

## For comparison: These are the numbers I got when I used a Mutex:

-> Read performance with the RwLock is better, but write performance is not as good as the Mutex.

```
redis-benchmark -t set,get, -n 100000 -q
SET: 145560.41 requests per second, p50=0.183 msec
GET: 142653.36 requests per second, p50=0.183 msec
```

## Actual redis benchmarks

```
redis-benchmark -t set,get, -n 100000 -q
SET: 166666.66 requests per second, p50=0.159 msec
GET: 175131.36 requests per second, p50=0.159 msec
```
