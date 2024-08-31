use std::cmp;
use std::time::{Duration, SystemTime};

#[cfg(test)]
use std::thread;

/// Percision of 5ms for take
pub struct TokenBucket {
    last_refreshed: SystemTime,
    max_refresh_duration: Duration,
    refresh_interval: Duration,
}
impl TokenBucket {
    pub fn new(refresh_interval_ms: u64, max_capacity: u64, initial_capacity: u64) -> TokenBucket {
        let current_tokens_count = cmp::min(max_capacity, initial_capacity);
        let last_refreshed = SystemTime::now()
            .checked_sub(Duration::from_millis(
                refresh_interval_ms * current_tokens_count,
            ))
            .expect("clock might have moved forward");

        TokenBucket {
            max_refresh_duration: Duration::from_millis(refresh_interval_ms * max_capacity),
            refresh_interval: Duration::from_millis(refresh_interval_ms),
            last_refreshed,
        }
    }

    fn get_effective_last_refreshed(&self) -> Option<SystemTime> {
        Some(cmp::max(
            self.last_refreshed,
            SystemTime::now().checked_sub(self.max_refresh_duration)?,
        ))
    }
    fn get_next_refreshed_time(&self) -> Option<SystemTime> {
        let effective_last_refreshed = self.get_effective_last_refreshed()?;
        let new_last_refreshed = effective_last_refreshed + self.refresh_interval;
        Some(new_last_refreshed)
    }
    pub fn try_take(&mut self) -> Option<()> {
        let new_last_refreshed = self.get_next_refreshed_time()?;
        let _ = SystemTime::now().duration_since(new_last_refreshed).ok()?;
        self.last_refreshed = new_last_refreshed;
        Some(())
    }

    pub fn take(&mut self) -> Option<()> {
        let effective_last_refreshed = self.get_effective_last_refreshed()?;
        let new_last_refreshed = effective_last_refreshed + self.refresh_interval;
        match SystemTime::now().duration_since(new_last_refreshed) {
            Ok(_) => {
                self.last_refreshed = new_last_refreshed;
            }
            Err(e) => {
                std::thread::sleep(e.duration());
                self.last_refreshed = new_last_refreshed;
            }
        };
        Some(())
    }
}

#[cfg(test)]
mod test_try_take {
    use super::*;

    #[test]
    fn initializes_with_proper_tokens() {
        // needs to have min(max capacity , initial_capacity)
        let mut tb = TokenBucket::new(1, 1, 2);
        assert!(tb.try_take().is_some());
        assert!(tb.try_take().is_none());
    }

    #[test]
    fn can_take_all_initial() {
        let mut tb = TokenBucket::new(1, 2, 2);
        assert!(tb.try_take().is_some());
        assert!(tb.try_take().is_some());
        assert!(tb.try_take().is_none());
    }

    #[test]
    fn can_take_generated_tokens() {
        let mut tb = TokenBucket::new(100, 2, 1);
        assert!(tb.try_take().is_some());
        thread::sleep(Duration::from_millis(100));
        assert!(tb.try_take().is_some());
        assert!(tb.try_take().is_none());
    }
}
#[cfg(test)]
mod test_take {
    use super::*;

    #[test]
    fn can_take_all_initial() {
        let mut tb = TokenBucket::new(50, 3, 3);
        assert!(tb.take().is_some());
        assert!(tb.take().is_some());
        assert!(tb.take().is_some());
    }

    #[test]
    fn can_take_after_waiting() {
        let mut tb = TokenBucket::new(50, 2, 1);
        assert!(tb.take().is_some());
        let now = SystemTime::now();
        assert!(tb.take().is_some());
        let elapsed = now
            .elapsed()
            .expect("clock might have went backward")
            .as_millis();
        assert!(elapsed >= 50 && elapsed <= 55);
    }

    #[test]
    fn can_take_multiple_after_waiting() {
        let mut tb = TokenBucket::new(10, 2, 0);
        let now = SystemTime::now();
        for _ in 0..10{
            assert!(tb.take().is_some());
        }
        let elapsed = now
            .elapsed()
            .expect("clock might have went backward")
            .as_millis();
        let bound = 100;
        assert!(elapsed >= bound && elapsed <= bound+5);
    }

    #[test]
    fn can_take_generated_tokens() {
        let mut tb = TokenBucket::new(50, 2, 0);
        thread::sleep(Duration::from_millis(100));
        let now = SystemTime::now();
        assert!(tb.take().is_some());
        assert!(tb.take().is_some());
        let elapsed = now
            .elapsed()
            .expect("clock might have went backward")
            .as_millis();
        assert!(elapsed == 0);
    }
}
