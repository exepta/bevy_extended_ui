use std::collections::VecDeque;
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub static UI_ID_GENERATE: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));

pub static BODY_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));

pub static DIV_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));

pub static BUTTON_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));

/// A pool that manages reusable integer IDs for widgets.
/// It hands out new IDs or recycles freed IDs.
pub struct IdPool {
    /// The next ID to assign if no free ID exists.
    counter: usize,
    /// Queue of IDs that have been released and can be reused.
    free_list: VecDeque<usize>,
}

impl IdPool {
    /// Creates a new empty `IdPool`.
    pub fn new() -> Self {
        Self {
            counter: 0,
            free_list: VecDeque::new(),
        }
    }

    /// Acquires a new ID.
    /// Returns a recycled ID if available, otherwise generates a new one.
    pub fn acquire(&mut self) -> usize {
        if let Some(id) = self.free_list.pop_front() {
            id
        } else {
            let id = self.counter;
            self.counter += 1;
            id
        }
    }

    /// Releases an ID back to the pool for reuse.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to release.
    pub fn release(&mut self, id: usize) {
        self.free_list.push_back(id);
    }
}