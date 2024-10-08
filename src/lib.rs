pub mod fifo;
pub mod fifo_reinserion;
pub mod ghost_fifo;

pub use fifo::FIFO;
pub use fifo_reinserion::FIFOReinsertion;
pub use ghost_fifo::GhostFIFO;
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
}
