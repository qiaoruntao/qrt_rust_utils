use std::collections::VecDeque;
use chrono::{DateTime, Duration, Local};

#[derive(Debug)]
pub struct TimedCounter {
    times: VecDeque<DateTime<Local>>,
    duration: Duration,
}

impl TimedCounter {
    pub fn create(duration: Duration) -> TimedCounter {
        TimedCounter {
            times: Default::default(),
            duration,
        }
    }

    pub fn incr(&mut self) {
        self.times.push_back(Local::now());
    }

    pub fn cnt(&mut self) -> usize {
        self.pop_expired();
        self.times.len()
    }

    fn pop_expired(&mut self) {
        while let Some(first) = self.times.get(0) {
            if Local::now() - *first > self.duration {
                self.times.pop_front();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut counter = TimedCounter::create(Duration::seconds(1));
        assert_eq!(counter.cnt(), 0);
        counter.incr();
        counter.incr();
        assert_eq!(counter.cnt(), 2);
        std::thread::sleep(core::time::Duration::from_secs(1));
        assert_eq!(counter.cnt(), 0);
    }
}
