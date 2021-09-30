use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc, thread::ThreadId, time};

use parking_lot::{Condvar, Mutex};

pub trait Delayed: Ord {
    fn delayed(&self) -> i64;
}

#[derive(Default)]
pub struct DelayQueue<T: Delayed> {
    queue: Arc<Mutex<DelayQueueInner<T>>>,
    available: Arc<Condvar>,
}

impl<T: Delayed> Clone for DelayQueue<T> {
    fn clone(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
            available: Arc::clone(&self.available),
        }
    }
}

#[derive(Default, Clone)]
struct DelayQueueInner<T: Delayed> {
    queue: BinaryHeap<Reverse<Arc<T>>>,
    current_thread: Option<ThreadId>,
}

impl<T: Delayed> DelayQueueInner<T> {
    fn peek(&self) -> Option<&T> {
        let result = self.queue.peek()?;
        Some(&result.0)
    }
}

impl<T> DelayQueue<T>
where
    T: Delayed + Sync + Send,
{
    pub fn put(&mut self, t: T) {
        let queue = self.queue.clone();
        let queue = &mut queue.lock().queue;
        let t = Reverse(Arc::new(t));
        queue.push(t.clone());
        if queue.peek() == Some(&t) {
            self.available.notify_one();
        }
    }

    pub fn take(&mut self) -> Arc<T> {
        let queue = self.queue.clone();
        let avaliable = self.available.clone();
        let mut guard = queue.lock();
        loop {
            match guard.peek() {
                None => {
                    avaliable.wait(&mut guard);
                }
                Some(first) => {
                    let delayed = first.delayed();
                    if delayed <= 0 {
                        let result = guard.queue.pop().unwrap();
                        if guard.current_thread.is_none() && guard.peek().is_some() {
                            avaliable.notify_one();
                        }
                        return result.0;
                    }
                    let _ = first;
                    match guard.current_thread {
                        Some(_) => {
                            avaliable.wait(&mut guard);
                        }
                        None => {
                            let thread_id = std::thread::current().id();
                            guard.current_thread = Some(thread_id);
                            avaliable
                                .wait_for(&mut guard, time::Duration::from_nanos(delayed as u64));
                            if guard.current_thread == Some(thread_id) {
                                guard.current_thread = None
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use chrono::{DateTime, Duration, Local};

    use super::*;
    #[test]
    fn test() {
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
                        // assert!(diff < 500);
                        // println!(
                        //     "thread: {:2}. {} task: {:?} diff: {}us",
                        //     thead_id, now, task, diff
                        // );
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
                "{:16}\t{:5}\t{:.2}%",
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
}
