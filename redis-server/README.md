Challenge Link -> https://codingchallenges.fyi/challenges/challenge-redis

## Commands handled

They can be seen at the `cmd` directory as well:

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

## My benchmark results after building with the release flag

```
redis-benchmark -t SET,GET -q
SET: 139275.77 requests per second, p50=0.191 msec
GET: 149925.03 requests per second, p50=0.175 msec
```

```
redis-benchmark -t set,get, -n 100000 -q
SET: 143061.52 requests per second, p50=0.183 msec
GET: 150375.94 requests per second, p50=0.175 msec
```

## Actual redis benchmarks

```
redis-benchmark -t SET,GET -q
SET: 167785.23 requests per second, p50=0.159 msec
GET: 172117.05 requests per second, p50=0.159 msec
```

```
redis-benchmark -t set,get, -n 100000 -q
SET: 166666.66 requests per second, p50=0.159 msec
GET: 175131.36 requests per second, p50=0.159 msec
```
