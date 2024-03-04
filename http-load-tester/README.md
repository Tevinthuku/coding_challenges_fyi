### How to run

```
cargo run -- -u http://localhost:8000 -n 20 -c 10
```

### Example output

```
Results:
 Total Requests (2XX).......................: 200
 Failed Requests (4XX)...................: 0
 Failed Requests (5XX)...................: 0
 Total Request Time (s) (Min, Max, Mean).....: 0.16, 5.72, 1.60
 Time to First Byte (s) (Min, Max, Mean).....: 0.11, 3.52, 1.25
 Time to Last Byte (s) (Min, Max, Mean)......: 0.16, 5.72, 1.60
```
