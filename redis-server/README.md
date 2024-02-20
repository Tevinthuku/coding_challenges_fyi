Challenge Link -> https://codingchallenges.fyi/challenges/challenge-redis

## My benchmark results after building with the release flag

```
redis-benchmark -t SET,GET -q
SET: 138504.16 requests per second, p50=0.183 msec
GET: 145348.83 requests per second, p50=0.175 msec
```

```
redis-benchmark -t set,get, -n 100000 -q
SET: 145560.41 requests per second, p50=0.183 msec
GET: 142653.36 requests per second, p50=0.183 msec
```

Also inspired by https://github.com/tokio-rs/mini-redis/
