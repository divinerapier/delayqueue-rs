use std::collections::HashMap;

use chrono::{DateTime, Duration, Local};
use delayqueue::{DelayQueue, Delayed};

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

const TOTAL_COUNT: usize = 1000;
const THREAD_COUNT: usize = 8;

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

    let maps = (0..THREAD_COUNT)
        .map(|_thead_id| {
            let mut queue = queue.clone();
            std::thread::spawn(move || {
                let mut map = HashMap::<i32, i32>::new();
                for _i in 0..TOTAL_COUNT / THREAD_COUNT {
                    let task = queue.take();
                    let now = chrono::Local::now();
                    let diff = (now.timestamp_nanos() - task.deadline) / 1000;
                    if diff <= 100 {
                        *map.entry(100).or_default() += 1;
                    } else if diff <= 200 {
                        *map.entry(200).or_default() += 1;
                    } else if diff <= 300 {
                        *map.entry(300).or_default() += 1;
                    } else if diff <= 400 {
                        *map.entry(400).or_default() += 1;
                    } else if diff <= 500 {
                        *map.entry(500).or_default() += 1;
                    } else if diff <= 600 {
                        *map.entry(600).or_default() += 1;
                    } else {
                        *map.entry(1000).or_default() += 1;
                    }
                }
                map
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>();

    let mut results = HashMap::<i32, i32>::default();

    for map in maps {
        for (k, v) in map {
            *results.entry(k).or_default() += v;
        }
    }

    let mut result_count = 0;
    println!("Response Latency\tCount\tPercent");
    for (k, v) in results {
        result_count += v;
        println!(
            "{:14}us\t{:5}\t{:.2}%",
            k,
            v,
            (v as f64 / TOTAL_COUNT as f64) * 100f64
        );
    }
    assert_eq!(1000, result_count);
}

fn after(du: Duration) -> DateTime<Local> {
    chrono::Local::now() + du
}
