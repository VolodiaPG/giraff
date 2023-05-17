use crossbeam_queue::ArrayQueue;

pub struct Pool<T: Default> {
    pool: ArrayQueue<T>,
}

impl<T: Default> Pool<T> {
    pub fn new(size: usize) -> Self { Self { pool: ArrayQueue::new(size) } }

    pub fn get(&self) -> T {
        match self.pool.pop() {
            Some(buffer) => buffer,
            None => T::default(),
        }
    }

    pub fn put(&self, buffer: T) { self.pool.force_push(buffer); }
}
