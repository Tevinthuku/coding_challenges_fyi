## Running the server

```bash
cargo run
```

you can also set the cache size. ie: The max number of bytes the server can hold. The default is `1000 bytes` if the `CACHE_SIZE` flag is not provided.

```bash
CACHE_SIZE=2000 cargo run
```

In a new terminal, connect to the server via telnet:

```bash
telnet localhost 11211
```

Passing some commands:

set a value

```bash
set test 0 0 4
1234
```

get a value

```bash
get test
```
