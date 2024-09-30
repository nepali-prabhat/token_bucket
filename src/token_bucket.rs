use std::cmp;
use std::fmt;
use std::time::{Duration, Instant};

#[cfg(test)]
use std::thread;

/// Percision of 5ms for take
#[derive(Clone, Copy)]
pub struct TokenBucket {
    last_refreshed: Instant,
    max_refresh_duration: Duration,
    refresh_interval: Duration,
}
impl TokenBucket {
    pub fn new(
        refresh_interval_ms: u64,
        max_capacity: u64,
        initial_capacity: u64,
    ) -> Option<TokenBucket> {
        if refresh_interval_ms == 0 {
            return None;
        }

        let current_tokens_count = cmp::min(max_capacity, initial_capacity);
        let last_refreshed = Instant::now().checked_sub(Duration::from_millis(
            refresh_interval_ms * current_tokens_count,
        ))?;

        Some(TokenBucket {
            max_refresh_duration: Duration::from_millis(refresh_interval_ms * max_capacity),
            refresh_interval: Duration::from_millis(refresh_interval_ms),
            last_refreshed,
        })
    }

    fn get_effective_last_refreshed(&self) -> Option<Instant> {
        Some(cmp::max(
            self.last_refreshed,
            Instant::now().checked_sub(self.max_refresh_duration)?,
        ))
    }
    fn get_next_refreshed_time(&self) -> Option<Instant> {
        let effective_last_refreshed = self.get_effective_last_refreshed()?;
        let new_last_refreshed = effective_last_refreshed + self.refresh_interval;
        Some(new_last_refreshed)
    }
    pub fn try_take(&mut self) -> Option<()> {
        let new_last_refreshed = self.get_next_refreshed_time()?;
        let _ = Instant::now()
            .checked_duration_since(new_last_refreshed)?;
        self.last_refreshed = new_last_refreshed;
        Some(())
    }

    pub fn take(&mut self) -> Option<()> {
        let effective_last_refreshed = self.get_effective_last_refreshed()?;
        let new_last_refreshed = effective_last_refreshed + self.refresh_interval;
        if let None = Instant::now().checked_duration_since(new_last_refreshed) {
                std::thread::sleep(new_last_refreshed.duration_since(Instant::now()));
        };
        self.last_refreshed = new_last_refreshed;
        Some(())
    }
}

// TODO: write tests
impl fmt::Debug for TokenBucket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.get_effective_last_refreshed() {
            Some(last_refreshed) => {
                let elapsed = Instant::now()
                    .checked_duration_since(last_refreshed)
                    .ok_or(fmt::Error)?;
                let count = elapsed
                    .as_millis()
                    .checked_div(self.refresh_interval.as_millis())
                    .or(Some(0));
                f.debug_tuple("TokenBucket").field(&count).finish()
            }
            None => Err(fmt::Error),
        }
    }
}

#[cfg(test)]
mod test_try_take {
    use super::*;

    #[test]
    fn initializes_with_proper_tokens() {
        // needs to have min(max capacity , initial_capacity)
        let mut tb = TokenBucket::new(1, 1, 2).unwrap();
        assert!(tb.try_take().is_some());
        assert!(tb.try_take().is_none());
    }

    #[test]
    fn can_take_all_initial() {
        let mut tb = TokenBucket::new(1, 2, 2).unwrap();
        assert!(tb.try_take().is_some());
        assert!(tb.try_take().is_some());
        assert!(tb.try_take().is_none());
    }

    #[test]
    fn can_take_generated_tokens() {
        let mut tb = TokenBucket::new(100, 2, 1).unwrap();
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
        let mut tb = TokenBucket::new(50, 3, 3).unwrap();
        assert!(tb.take().is_some());
        assert!(tb.take().is_some());
        assert!(tb.take().is_some());
    }

    #[test]
    fn can_take_after_waiting() {
        let mut tb = TokenBucket::new(50, 2, 1).unwrap();
        assert!(tb.take().is_some());
        let now = Instant::now();
        assert!(tb.take().is_some());
        let elapsed = now.elapsed().as_millis();
        assert!(elapsed >= 50 && elapsed <= 55);
    }

    #[test]
    fn can_take_multiple_after_waiting() {
        let mut tb = TokenBucket::new(10, 2, 0).unwrap();
        let now = Instant::now();
        for _ in 0..10 {
            assert!(tb.take().is_some());
        }
        let elapsed = now.elapsed().as_millis();
        let bound = 100;
        assert!(elapsed >= bound && elapsed <= bound + 5);
    }

    #[test]
    fn can_take_generated_tokens() {
        let mut tb = TokenBucket::new(50, 2, 0).unwrap();
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        assert!(tb.take().is_some());
        assert!(tb.take().is_some());
        let elapsed = now.elapsed().as_millis();
        assert!(elapsed == 0);
    }
}
