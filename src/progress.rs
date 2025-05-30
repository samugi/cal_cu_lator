use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct Progress {
    total: u32,
    current: Arc<AtomicU32>,
    last_percent: Arc<AtomicU32>,
}

impl Progress {
    pub fn new(total: u32) -> Self {
        Progress {
            total,
            current: Arc::new(AtomicU32::new(0)),
            last_percent: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn tick(&self) {
        let done = self.current.fetch_add(1, Ordering::Relaxed) + 1;
        let percent = (done * 100 / self.total).min(100);
        let last = self.last_percent.load(Ordering::Relaxed);

        if percent > last && self
                .last_percent
                .compare_exchange(last, percent, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok() {
            println!("{}%", percent);
        }
    }
}
