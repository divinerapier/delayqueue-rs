# DelayQueue

## Usage

Add this to your Cargo.toml:

[dependencies]
delayqueue = "0.0.2"

### Delayed Task

``` rust
#[derive(Default, Debug, PartialEq, Eq)]
struct Task {
    deadline: i64,

    message: String,
}

impl Task {
    fn new<S: Into<String>>(deadline: i64, message: S) -> Task {
        let message = message.into();
        Task { deadline, message }
    }
}

impl Delayed for Task {
    fn delayed(&self) -> i64 {
        self.deadline - chrono::Local::now().timestamp_nanos()
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deadline.cmp(&other.deadline)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.deadline.cmp(&other.deadline))
    }
}
```

### Producer

``` rust
fn main() {
    let queue = DelayQueue::<Task>::default();
    {
        let mut queue = queue.clone();
        std::thread::spawn(move || {
            for index in 0..TOTAL_COUNT {
                let v = rand::random::<u64>() % 10000;
                queue.put(Task::new(
                    after(Duration::milliseconds(v as i64)).timestamp_nanos(),
                    format!("index: {}. delay for {}ms", index, v),
                ));
            }
        });
    }
}
```

### Consumer

``` rust
fn main() {
    let queue = DelayQueue::<Task>::default();
    {
        let mut queue = queue.clone();
        std::thread::spawn(move || {
            let task = queue.take();
        });
    }
}
```

## Unit Test

``` bash
$ cargo test -- --nocapture

Response Latency        Count   Percent
           100us           16   1.60%
           200us          251   25.10%
           300us          244   24.40%
           400us          123   12.30%
           500us           46   4.60%
           600us           42   4.20%
          1000us          278   27.80%
test test::test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.99s
```

## Examples

``` bash
$ cargo run --example=simple
```
