use std::sync::Mutex;

/// Buffers items and hands back a full batch once `size` is reached, so
/// callers can emit fewer, larger events instead of one per item. Needed
/// at scale: `invoke()` and `emit`/`listen` are separate channels with no
/// ordering guarantee, and individual events piling up faster than the
/// frontend can process them can look like silent drops. Thread-safe -
/// multiple threads can push concurrently.
pub struct Batcher<T> {
    buf: Mutex<Vec<T>>,
    size: usize,
}

impl<T> Batcher<T> {
    pub fn new(size: usize) -> Self {
        Self { buf: Mutex::new(Vec::new()), size }
    }

    /// Adds `item`; returns a full batch if the buffer just reached
    /// capacity, otherwise `None`.
    pub fn push(&self, item: T) -> Option<Vec<T>> {
        let mut buf = self.buf.lock().unwrap();
        buf.push(item);
        if buf.len() >= self.size {
            Some(std::mem::take(&mut *buf))
        } else {
            None
        }
    }

    /// Drains and returns whatever's left (fewer than `size` items).
    pub fn flush(&self) -> Vec<T> {
        std::mem::take(&mut *self.buf.lock().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_until_full() {
        let batcher = Batcher::new(3);
        assert!(batcher.push(1).is_none());
        assert!(batcher.push(2).is_none());
        assert_eq!(batcher.push(3), Some(vec![1, 2, 3]));
    }

    #[test]
    fn resets_after_a_full_batch() {
        let batcher = Batcher::new(2);
        assert_eq!(batcher.push(1), None);
        assert_eq!(batcher.push(2), Some(vec![1, 2]));
        assert_eq!(batcher.push(3), None);
        assert_eq!(batcher.flush(), vec![3]);
    }

    #[test]
    fn flush_on_empty_buffer_returns_empty() {
        let batcher: Batcher<i32> = Batcher::new(5);
        assert_eq!(batcher.flush(), Vec::<i32>::new());
    }
}
