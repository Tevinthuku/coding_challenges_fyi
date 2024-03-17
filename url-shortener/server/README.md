## How to run the app

1. Run the postgres docker container

```
docker-compose up -d
```

2. Run the server

```
DATABASE_URL=postgres://postgres:postgres@localhost/postgres cargo run
```

3. Shorten a URL

```
POST localhost:5000/shorten

JSON body:

{
	"url": "https://github.com/launchbadge/sqlx/blob/main/examples/postgres/todos/src/main.rs"
}
```

4. Get long url / redirect

```
GET localhost:5000/ODNlNDc
```

5. Delete URL

```
DELETE localhost:5000/ODNlNDc
```
