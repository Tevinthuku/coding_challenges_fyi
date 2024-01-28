## Running the load balancer and servers.

## Running the servers

```
PORT=<your preffered port> cargo run --bin backend
```

```
PORT=8080 cargo run --bin backend
```

```
PORT=8081 cargo run --bin backend
```

When you want to run more servers, be sure to add them to the `src/bin/load_balancer/servers.toml` file.

## Running the load balancer

```
HEALTH_CHECK_INTERVAL=<number_in_seconds> cargo run --bin load_balancer
```

`number_in_seconds` defaults to 1.

```
HEALTH_CHECK_INTERVAL=1 cargo run --bin load_balancer
```

Challenge URL: https://codingchallenges.fyi/challenges/challenge-load-balancer

Curl testing

```
curl --parallel --parallel-immediate --parallel-max 6 --config urls.txt
```
