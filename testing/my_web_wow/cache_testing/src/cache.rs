use std::collections::HashMap;

/// A little container holding your object plus a usage counter.
struct CacheContainer<T> {
    obj: T,
    counter: usize,
}

/// A simple cache: on `get(key)`, we either bump-and-clone the existing
/// entry, or call your `load(key)` → `process(...)` functions and store it.
pub struct Cache<T, L, P>
where
    T: Clone,
    L: Fn(&str) -> T,
    P: Fn(T) -> T,
{
    load_fn:    L,
    process_fn: P,
    cache:      HashMap<String, CacheContainer<T>>,
}

impl<T, L, P> Cache<T, L, P>
where
    T: Clone,
    L: Fn(&str) -> T,
    P: Fn(T) -> T,
{
    /// Create a new cache. Both closures are required.
    pub fn new(load_fn: L, process_fn: P) -> Self {
        Cache {
            load_fn,
            process_fn,
            cache: HashMap::new(),
        }
    }

    /// Get a value: if cached, bump its counter and return a `T` clone.
    /// Otherwise load, process, insert (counter=1) and return it.
    pub fn get(&mut self, key: &str) -> T {
        if let Some(container) = self.cache.get_mut(key) {
            container.counter += 1;
            return container.obj.clone();
        }
        self.load_and_process(key)
    }

    /// Like `get`, but *only* returns an existing value (never loads).
    pub fn get_cached(&mut self, key: &str) -> Option<T> {
        if let Some(container) = self.cache.get_mut(key) {
            container.counter += 1;
            Some(container.obj.clone())
        } else {
            None
        }
    }

    /// Insert (or overwrite) with counter = 1.
    pub fn put(&mut self, key: &str, obj: T) {
        self.cache.insert(
            key.to_string(),
            CacheContainer { obj, counter: 1 },
        );
    }

    /// Decrement the counter; if it hits zero, remove the entry
    /// (and let Rust’s `Drop` do any cleanup).
    pub fn remove(&mut self, key: &str) {
        // do the decrement in a sub‐scope so the mutable borrow ends before removal
        let maybe_new_count = {
            if let Some(container) = self.cache.get_mut(key) {
                container.counter = container.counter.saturating_sub(1);
                Some(container.counter)
            } else {
                None
            }
        };
        if let Some(0) = maybe_new_count {
            // removes the entry; its `obj` is dropped here
            self.cache.remove(key);
        }
    }

    fn load_and_process(&mut self, key: &str) -> T {
        // if either closure panics, it propagates
        let loaded = (self.load_fn)(key);
        let processed = (self.process_fn)(loaded.clone());
        // store a clone, in case user‐closures keep ownership of `processed`
        self.cache.insert(
            key.to_string(),
            CacheContainer {
                obj: processed.clone(),
                counter: 1,
            },
        );
        processed
    }
}
