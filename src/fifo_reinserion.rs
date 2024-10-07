use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
struct Item<V> {
    value: V,
    weight: usize,
    hit: bool,
    removed: bool,
}

#[derive(Debug)]
pub struct FIFOReinsertion<K, V> {
    hash: HashMap<K, Item<V>>,
    vec_deque: VecDeque<K>,
    used_capacity: usize,
    capacity: usize,
}

#[derive(Debug)]
pub enum FIFOReinsertionError {
    BeyondCapacity,
}

type RemovedKeys<K> = Vec<K>;

impl<K, V> FIFOReinsertion<K, V>
where
    K: Eq + Hash + Copy + Debug,
    V: Debug,
{
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        FIFOReinsertion {
            hash: HashMap::new(),
            vec_deque: VecDeque::new(),
            used_capacity: 0,
            capacity,
        }
    }

    pub fn get(&mut self, key: K) -> Option<&V> {
        if let Some(item) = self.hash.get_mut(&key) {
            if item.removed {
                return None;
            }

            item.hit = true;
            Some(&item.value)
        } else {
            None
        }
    }

    fn update(&mut self, key: K, value: V, weight: usize) -> Option<RemovedKeys<K>> {
        let item = self.hash.get_mut(&key).unwrap();
        item.value = value;
        let old_weight = item.weight;
        item.weight = weight;
        item.removed = false;

        if weight > old_weight {
            let needed_space = weight - old_weight;
            let removed_keys = self.free(needed_space, Some(key));
            self.used_capacity += needed_space;
            removed_keys
        } else {
            self.used_capacity -= old_weight - weight;
            None
        }
    }

    fn insert(&mut self, key: K, value: V, weight: usize) -> Option<RemovedKeys<K>> {
        let removed_keys = self.free(weight, None);
        self.used_capacity += weight;
        self.hash.insert(
            key,
            Item {
                value,
                weight,
                hit: false,
                removed: false,
            },
        );
        self.vec_deque.push_back(key);

        removed_keys
    }

    //
    /// # Errors
    ///
    /// Returns `CacheError::BeyondCapacity` if the weight is greater than the capacity.
    pub fn put(
        &mut self,
        key: K,
        value: V,
        weight: usize,
    ) -> Result<Option<RemovedKeys<K>>, FIFOReinsertionError> {
        if weight > self.capacity {
            return Err(FIFOReinsertionError::BeyondCapacity);
        }

        if self.hash.contains_key(&key) {
            Ok(self.update(key, value, weight))
        } else {
            Ok(self.insert(key, value, weight))
        }
    }

    fn free(&mut self, weight: usize, ignore_key: Option<K>) -> Option<RemovedKeys<K>> {
        let mut removed_keys = vec![];
        while self.used_capacity + weight > self.capacity {
            let key = self.vec_deque.pop_front().unwrap();
            let item = self.hash.get_mut(&key).unwrap();

            if item.removed {
                self.used_capacity -= item.weight;
                self.hash.remove(&key);
                continue;
            }

            if Some(key) == ignore_key {
                self.vec_deque.push_back(key);
                continue;
            }

            if item.hit {
                self.vec_deque.push_back(key);
                item.hit = false;
                continue;
            }

            self.used_capacity -= item.weight;
            self.hash.remove(&key);
            removed_keys.push(key);
        }

        if removed_keys.is_empty() {
            None
        } else {
            Some(removed_keys)
        }
    }

    pub fn remove(&mut self, key: K) {
        let item = self.hash.get_mut(&key);

        if let Some(item) = item {
            item.removed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut cache = FIFOReinsertion::new(10);
        cache.put(1, 1, 2).unwrap();
        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);

        assert_eq!(cache.used_capacity, 2);
        assert_eq!(cache.capacity, 10);
    }

    #[test]
    fn it_should_free_space() {
        let mut cache = FIFOReinsertion::new(10);
        cache.put(1, 1, 2).unwrap();
        cache.put(2, 2, 3).unwrap();
        cache.put(3, 3, 4).unwrap();
        cache.put(4, 4, 1).unwrap();

        cache.free(5, None);

        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some(&3));
        assert_eq!(cache.get(4), Some(&4));

        assert_eq!(cache.used_capacity, 5);
    }

    #[test]
    fn it_should_remove() {
        let mut cache = FIFOReinsertion::new(10);
        cache.put(1, 1, 2).unwrap();
        cache.put(2, 2, 3).unwrap();
        cache.put(3, 3, 4).unwrap();
        cache.put(4, 4, 1).unwrap();

        cache.remove(2);

        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some(&3));
        assert_eq!(cache.get(4), Some(&4));

        assert_eq!(cache.used_capacity, 10);
    }

    #[test]
    fn it_should_hit() {
        let mut cache = FIFOReinsertion::new(10);
        cache.put(1, 1, 2).unwrap();
        cache.put(2, 2, 3).unwrap();
        cache.put(3, 3, 4).unwrap();
        cache.put(4, 4, 1).unwrap();

        cache.get(1);

        cache.put(5, 5, 5).unwrap();

        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), None);
        assert_eq!(cache.get(4), Some(&4));
        assert_eq!(cache.get(5), Some(&5));

        assert_eq!(cache.used_capacity, 8);
    }

    #[test]
    #[should_panic = "BeyondCapacity"]
    fn it_should_panic() {
        let mut cache = FIFOReinsertion::new(10);
        cache.put(1, 1, 2).unwrap();
        cache.put(2, 2, 3).unwrap();
        cache.put(3, 3, 4).unwrap();
        cache.put(4, 4, 1).unwrap();

        cache.put(5, 5, 11).unwrap();
    }

    #[test]
    fn it_should_update() {
        let mut cache = FIFOReinsertion::new(10);
        cache.put(1, 1, 2).unwrap();
        cache.put(2, 2, 3).unwrap();
        cache.put(3, 3, 4).unwrap();
        cache.put(4, 4, 1).unwrap();

        cache.put(1, 10, 3).unwrap();

        assert_eq!(cache.get(1), Some(&10));
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some(&3));
        assert_eq!(cache.get(4), Some(&4));

        assert_eq!(cache.used_capacity, 8);
    }

    #[test]
    fn it_should_update_to_lower_weight() {
        let mut cache = FIFOReinsertion::new(10);
        cache.put(1, 1, 3).unwrap();
        cache.put(2, 2, 2).unwrap();
        cache.put(3, 3, 4).unwrap();
        cache.put(4, 4, 1).unwrap();

        cache.put(1, 10, 2).unwrap();

        assert_eq!(cache.get(1), Some(&10));
        assert_eq!(cache.get(2), Some(&2));
        assert_eq!(cache.get(3), Some(&3));
        assert_eq!(cache.get(4), Some(&4));

        assert_eq!(cache.used_capacity, 9);
    }

    #[test]
    fn it_should_remove_removed_key() {
        let mut cache = FIFOReinsertion::new(2);

        cache.put(1, 1, 1).unwrap();
        cache.remove(1);
        cache.put(2, 2, 2).unwrap();

        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), Some(&2));
        assert_eq!(cache.vec_deque.len(), 1);
        assert_eq!(cache.hash.len(), 1);
        assert_eq!(cache.used_capacity, 2);
    }

    #[test]
    fn it_should_remove_removed_key_2() {
        let mut cache = FIFOReinsertion::new(3);

        cache.put(1, 1, 1).unwrap();
        cache.remove(1);
        cache.put(2, 2, 2).unwrap();
        cache.put(3, 3, 1).unwrap();

        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), Some(&2));
        assert_eq!(cache.get(3), Some(&3));

        assert_eq!(cache.vec_deque.len(), 2);
        assert_eq!(cache.hash.len(), 2);
        assert_eq!(cache.used_capacity, 3);
    }

    #[test]
    fn it_should_return_removed_key() {
        let mut cache = FIFOReinsertion::new(3);

        cache.put(1, 1, 1).unwrap();
        cache.put(2, 2, 2).unwrap();

        let removed_keys = cache.put(3, 3, 1).unwrap().unwrap();

        assert_eq!(removed_keys, vec![1]);
        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), Some(&2));
        assert_eq!(cache.get(3), Some(&3));
        assert_eq!(cache.get(4), None);

        assert_eq!(cache.vec_deque.len(), 2);
        assert_eq!(cache.hash.len(), 2);
        assert_eq!(cache.used_capacity, 3);
    }
}
