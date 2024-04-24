## How to run the 2 servers plus a rate limiter:

### Running the dragonfly backed sliding window rate limiter

```bash
docker-compose up -d
```

### Run first server

```bash
cargo run --bin server_1
```

### Run second server

```
cargo run --bin server_2
```


- Using postman or Insomnia, call this endpoint until it returns 429 (Automate the requests sent (60 to be exact)) `localhost:8080/limited`

- Then call this endpoint `localhost:8081/limited` and you will still get `429`
