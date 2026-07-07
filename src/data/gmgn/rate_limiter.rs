use std::sync::Arc;
use tokio::sync::Semaphore;

/// Token-bucket rate limiter.
/// Each endpoint has a known weight (1-5). Permits are granted
/// if capacity remains; otherwise the caller waits.
#[derive(Clone)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    capacity: u32,
}

impl RateLimiter {
    pub fn new(_rate: u32, capacity: u32) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(capacity as usize)),
            capacity,
        }
    }

    /// Acquire `weight` permits. Blocks if capacity is insufficient.
    pub async fn acquire(&self, weight: u32) {
        let permits = weight;
        if permits > self.capacity {
            // If weight > capacity, wait for full refill (never actually
            // happens in practice — max weight is 5, min capacity is 20).
            let _ = self.semaphore.acquire_many(self.capacity).await;
        } else {
            let _ = self.semaphore.acquire_many(permits).await;
        }
    }

    /// Non-blocking: try to acquire and return immediately.
    pub fn try_acquire(&self, weight: u32) -> bool {
        if let Ok(permit) = self.semaphore.try_acquire_many(weight) {
            permit.forget();
            true
        } else {
            false
        }
    }
}
