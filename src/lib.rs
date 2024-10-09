mod fifo;
mod fifo_reinserion;
mod ghost_fifo;

use fifo::FIFOError;
use fifo::FIFO;
use fifo_reinserion::FIFOReinsertion;
use fifo_reinserion::FIFOReinsertionError;
use ghost_fifo::GhostFIFO;

use std::fmt::Debug;
use std::hash::Hash;

pub struct S3FIFO<K, V> {
    main: FIFOReinsertion<K, V>,
    small: FIFO<K, V>,
    ghost: GhostFIFO<K>,
}

#[derive(Debug)]
pub enum S3FIFOError {
    BeyondCapacity,
}

impl<K, V> S3FIFO<K, V>
where
    K: Eq + Hash + Debug + Copy,
    V: Clone + Debug,
{
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let main_capacity = capacity * 90 / 100;
        let small_capacity = capacity * 10 / 100;
        Self {
            main: FIFOReinsertion::new(main_capacity),
            small: FIFO::new(small_capacity),
            ghost: GhostFIFO::new(main_capacity),
        }
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if the cache is beyond capacity of small fifo.
    pub fn put(&mut self, key: K, value: V, weight: usize) -> Result<Option<Vec<K>>, S3FIFOError> {
        if self.ghost.get(key) {
            self.ghost.remove(key);
            match self.main.put(key, value, weight) {
                Err(FIFOReinsertionError::BeyondCapacity) => Err(S3FIFOError::BeyondCapacity),
                Ok(removed) => Ok(removed),
            }
        } else {
            match self.small.put(key, value, weight) {
                Err(FIFOError::BeyondCapacity) => Err(S3FIFOError::BeyondCapacity),
                Ok(removed) => match removed {
                    Some(removed) => {
                        let mut removed_keys = vec![];
                        for item in removed {
                            if item.freq > 0 {
                                if let Ok(Some(removed_from_main)) = self.main.put_with_freq(
                                    item.key,
                                    item.value,
                                    item.weight,
                                    item.freq - 1,
                                ) {
                                    removed_keys.extend(removed_from_main);
                                }
                            } else {
                                let _ = self.ghost.put(item.key, item.weight);
                                removed_keys.push(item.key);
                            }
                        }

                        Ok(Some(removed_keys))
                    }
                    None => Ok(None),
                },
            }
        }
    }

    pub fn get(&mut self, key: K) -> Option<&V> {
        self.small.get(key).or_else(|| self.main.get(key))
    }

    pub fn remove(&mut self, key: K) {
        self.main.remove(key);
        self.small.remove(key);
        self.ghost.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_works() {
        let mut cache = FIFO::new(10);
        cache.put(1, 1, 2).unwrap();
        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);
    }

    #[test]
    fn fifo_reinserion() {
        let mut cache = FIFOReinsertion::new(10);
        cache.put(1, 1, 2).unwrap();
        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);
    }

    #[test]
    fn ghost_fifo() {
        let mut cache = GhostFIFO::new(10);
        cache.put(1, 2).unwrap();
        assert!(cache.get(1));
        assert!(!cache.get(2));
    }

    #[test]
    fn s3fifo_works() {
        let mut cache = S3FIFO::new(10);
        cache.put(1, 1, 1).unwrap();
        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);
    }

    #[test]
    fn it_should_has_removed_keys() {
        let mut cache = S3FIFO::new(10);
        cache.put(1, 1, 1).unwrap();
        cache.put(2, 2, 1).unwrap();
        cache.put(3, 3, 1).unwrap();
        cache.put(4, 4, 1).unwrap();
        cache.put(5, 5, 1).unwrap();
        cache.put(6, 6, 1).unwrap();
        cache.put(7, 7, 1).unwrap();
        cache.put(8, 8, 1).unwrap();
        cache.put(9, 9, 1).unwrap();
        cache.put(10, 10, 1).unwrap();
        let removed_keys = cache.put(11, 11, 1).unwrap();

        assert_eq!(removed_keys, Some(vec![10]));

        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), None);
        assert_eq!(cache.get(4), None);
        assert_eq!(cache.get(5), None);
        assert_eq!(cache.get(6), None);
        assert_eq!(cache.get(7), None);
        assert_eq!(cache.get(8), None);
        assert_eq!(cache.get(9), None);
        assert_eq!(cache.get(10), None);
        assert_eq!(cache.get(11), Some(&11));
    }

    #[test]
    fn it_should_have_something_in_main() {
        let mut cache = S3FIFO::new(10);
        cache.put(1, 1, 1).unwrap();
        cache.put(2, 2, 1).unwrap();
        cache.put(3, 3, 1).unwrap();
        cache.put(4, 4, 1).unwrap();
        cache.put(5, 5, 1).unwrap();
        cache.put(6, 6, 1).unwrap();
        cache.put(7, 7, 1).unwrap();
        cache.put(8, 8, 1).unwrap();
        cache.put(9, 9, 1).unwrap();
        cache.put(10, 10, 1).unwrap();
        cache.get(10);
        let removed_keys = cache.put(11, 11, 1).unwrap();

        assert_eq!(removed_keys, Some(vec![]));

        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), None);
        assert_eq!(cache.get(4), None);
        assert_eq!(cache.get(5), None);
        assert_eq!(cache.get(6), None);
        assert_eq!(cache.get(7), None);
        assert_eq!(cache.get(8), None);
        assert_eq!(cache.get(9), None);
        assert_eq!(cache.get(10), Some(&10));
        assert_eq!(cache.get(11), Some(&11));
    }

    #[test]
    #[should_panic = "BeyondCapacity"]
    fn it_should_panic() {
        let mut cache = S3FIFO::new(10);
        cache.put(1, 1, 2).unwrap();
    }
}
