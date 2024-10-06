#[derive(Debug)]
struct Item<V> {
    value: V,
    weight: usize,
}

#[derive(Debug)]
pub struct Cache<K, V> {
    hash: std::collections::HashMap<K, Item<V>>,
    vec_deque: std::collections::VecDeque<K>,
    used_capacity: usize,
    capacity: usize,
}

#[derive(Debug)]
pub enum CacheError {
    BeyondCapacity,
    CapacityExceeded,
    ItemNotFound,
    ItemExist,
}

impl<K, V> Cache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + Copy + std::fmt::Debug,
    V: std::fmt::Debug,
{
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Cache {
            hash: std::collections::HashMap::new(),
            vec_deque: std::collections::VecDeque::new(),
            used_capacity: 0,
            capacity,
        }
    }

    pub fn get(&mut self, key: K) -> Option<&V> {
        if let Some(item) = self.hash.get(&key) {
            Some(&item.value)
        } else {
            None
        }
    }

    //
    /// # Errors
    ///
    /// Returns `CacheError::BeyondCapacity` if the weight is greater than the capacity.
    /// Returns `CacheError::CapacityExceeded` if the cache is full.
    /// Returns `CacheError::ItemExist` if the key already exists.
    pub fn put(&mut self, key: K, value: V, weight: usize) -> Result<(), CacheError> {
        if weight > self.capacity {
            return Err(CacheError::BeyondCapacity);
        }

        if self.used_capacity + weight > self.capacity {
            return Err(CacheError::CapacityExceeded);
        }

        self.used_capacity += weight;

        if self.hash.contains_key(&key) {
            return Err(CacheError::ItemExist);
        }

        let item = Item { value, weight };

        self.hash.insert(key, item);
        self.vec_deque.push_back(key);

        Ok(())
    }

    pub fn free_space(&mut self, weight: usize) -> Vec<K> {
        let mut removed_keys = vec![];
        while self.used_capacity + weight > self.capacity {
            let key = self.vec_deque.pop_front().unwrap();
            let item = self.hash.get(&key).unwrap();
            self.used_capacity -= item.weight;
            self.hash.remove(&key);
            removed_keys.push(key);
        }

        removed_keys
    }

    pub fn remove(&mut self, key: K) -> Result<(), CacheError> {
        let item = self.hash.remove(&key);
        if item.is_none() {
            return Err(CacheError::ItemNotFound);
        }

        self.vec_deque.retain(|&x| x != key);

        self.used_capacity -= item.unwrap().weight;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut cache = Cache::new(10);
        cache.put(1, 1, 2).unwrap();
        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);
    }

    #[test]
    fn it_should_free_space() {
        let mut cache = Cache::new(10);
        cache.put(1, 1, 2).unwrap();
        cache.put(2, 2, 3).unwrap();
        cache.put(3, 3, 4).unwrap();
        cache.put(4, 4, 1).unwrap();

        cache.free_space(5);

        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some(&3));
        assert_eq!(cache.get(4), Some(&4));
    }

    #[test]
    fn it_should_remove() {
        let mut cache = Cache::new(10);
        cache.put(1, 1, 2).unwrap();
        cache.put(2, 2, 3).unwrap();
        cache.put(3, 3, 4).unwrap();
        cache.put(4, 4, 1).unwrap();

        cache.remove(2).unwrap();

        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some(&3));
        assert_eq!(cache.get(4), Some(&4));
    }
}
