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
redis-benchmark -t set,get, -n 100000 -q
SET: 137741.05 requests per second, p50=0.183 msec
GET: 149476.83 requests per second, p50=0.175 msec
```

## For comparison: These are the numbers I got when I used a Mutex:

-> Read performance with the RwLock is better, but write performance is not as good as the Mutex.

````
redis-benchmark -t set,get, -n 100000 -q
SET: 145560.41 requests per second, p50=0.183 msec
GET: 142653.36 requests per second, p50=0.183 msec
```

## Actual redis benchmarks

````

redis-benchmark -t set,get, -n 100000 -q
SET: 166666.66 requests per second, p50=0.159 msec
GET: 175131.36 requests per second, p50=0.159 msec

```


```
