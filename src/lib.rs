pub mod fifo_reinserion;

pub use fifo_reinserion::FIFOReinsertion;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_reinserion() {
        let mut cache = FIFOReinsertion::new(10);
        cache.put(1, 1, 2).unwrap();
        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);
    }
}
