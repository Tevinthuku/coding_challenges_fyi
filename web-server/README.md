Based off https://codingchallenges.fyi/challenges/challenge-webserver

## How to run the server:

We need to run this as the root, at least on mac (Idk about other OS) because we are running on port 80;

```
sudo cargo run
```

specifying a FILE_DIRECTORY. It defaults to "./www/" if the env var is not provided.

```
FILE_DIRECTORY="./www/" sudo cargo run
```

Sending requests

```
curl -i http://localhost/index.html
```

```
curl -i http://localhost/invalid.html
```
