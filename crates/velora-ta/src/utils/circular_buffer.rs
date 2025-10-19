//! Efficient circular buffer for windowed calculations.
//!
//! A circular buffer is a fixed-size buffer that overwrites old data when full.
//! This is ideal for indicators that only need a sliding window of recent data.

use std::ops::Add;

/// Fixed-size circular buffer optimized for indicator calculations.
///
/// This buffer maintains a fixed capacity and automatically overwrites the oldest
/// data when new data is added. It's more memory-efficient than keeping all
/// historical data when only a fixed window is needed.
///
/// # Example
///
/// ```ignore
/// let mut buffer = CircularBuffer::new(3);
/// buffer.push(10.0);
/// buffer.push(20.0);
/// buffer.push(30.0);  // Buffer is now full: [10, 20, 30]
///
/// assert_eq!(buffer.sum(), 60.0);
/// assert_eq!(buffer.mean(), Some(20.0));
///
/// buffer.push(40.0);  // Overwrites 10: [20, 30, 40]
/// assert_eq!(buffer.sum(), 90.0);
/// ```
#[derive(Debug, Clone)]
pub struct CircularBuffer<T> {
    data: Vec<T>,
    capacity: usize,
    head: usize,
    size: usize,
}

impl<T: Copy + Default> CircularBuffer<T> {
    /// Create a new circular buffer with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of elements the buffer can hold
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        Self {
            data: vec![T::default(); capacity],
            capacity,
            head: 0,
            size: 0,
        }
    }

    /// Add a new value to the buffer.
    ///
    /// If the buffer is full, this overwrites the oldest value.
    pub fn push(&mut self, value: T) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.capacity;
        if self.size < self.capacity {
            self.size += 1;
        }
    }

    /// Get a value at a specific index.
    ///
    /// Index 0 is the oldest value, index `size - 1` is the newest.
    /// Returns `None` if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<T> {
        if index >= self.size {
            return None;
        }
        let actual_index = (self.head + self.capacity - self.size + index) % self.capacity;
        Some(self.data[actual_index])
    }

    /// Get the most recent value (last pushed).
    pub fn last(&self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        let index = (self.head + self.capacity - 1) % self.capacity;
        Some(self.data[index])
    }

    /// Get the oldest value in the buffer.
    pub fn first(&self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        self.get(0)
    }

    /// Check if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Get the current number of elements in the buffer.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Get the maximum capacity of the buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all elements from the buffer.
    pub fn clear(&mut self) {
        self.head = 0;
        self.size = 0;
    }

    /// Iterate over all values in the buffer (oldest to newest).
    pub fn iter(&self) -> CircularBufferIter<'_, T> {
        CircularBufferIter {
            buffer: self,
            index: 0,
        }
    }

    /// Get a slice view of the buffer's data in insertion order.
    ///
    /// Note: This returns the values in chronological order (oldest to newest),
    /// which may not match the internal storage order.
    pub fn as_slice(&self) -> Vec<T> {
        self.iter().copied().collect()
    }
}

// Specialized methods for numeric types
impl<T> CircularBuffer<T>
where
    T: Copy + Default + Add<Output = T> + PartialOrd,
{
    /// Calculate the sum of all values in the buffer.
    pub fn sum(&self) -> T
    where
        T: Add<Output = T> + Default,
    {
        self.iter().fold(T::default(), |acc, &x| acc + x)
    }

    /// Find the maximum value in the buffer.
    ///
    /// Returns `None` if the buffer is empty.
    pub fn max(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        self.iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Find the minimum value in the buffer.
    ///
    /// Returns `None` if the buffer is empty.
    pub fn min(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        self.iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }
}

impl CircularBuffer<f64> {
    /// Calculate the mean (average) of all values in the buffer.
    ///
    /// Returns `None` if the buffer is empty.
    pub fn mean(&self) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        Some(self.sum() / self.size as f64)
    }

    /// Calculate the variance of all values in the buffer.
    ///
    /// Returns `None` if the buffer is empty.
    pub fn variance(&self) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        let mean = self.mean()?;
        let sum_squared_diff: f64 = self.iter().map(|&x| (x - mean).powi(2)).sum();
        Some(sum_squared_diff / self.size as f64)
    }

    /// Calculate the standard deviation of all values in the buffer.
    ///
    /// Returns `None` if the buffer is empty.
    pub fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }
}

/// Iterator over circular buffer elements (oldest to newest).
#[derive(Debug)]
pub struct CircularBufferIter<'a, T> {
    buffer: &'a CircularBuffer<T>,
    index: usize,
}

impl<'a, T: Copy> Iterator for CircularBufferIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.buffer.size {
            return None;
        }
        let actual_index = (self.buffer.head + self.buffer.capacity - self.buffer.size
            + self.index)
            % self.buffer.capacity;
        self.index += 1;
        Some(&self.buffer.data[actual_index])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.size - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T: Copy> ExactSizeIterator for CircularBufferIter<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer: CircularBuffer<f64> = CircularBuffer::new(5);
        assert_eq!(buffer.capacity(), 5);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
    }

    #[test]
    #[should_panic(expected = "Capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        let _buffer: CircularBuffer<f64> = CircularBuffer::new(0);
    }

    #[test]
    fn test_push_and_get() {
        let mut buffer = CircularBuffer::new(3);
        buffer.push(10.0);
        buffer.push(20.0);
        buffer.push(30.0);

        assert_eq!(buffer.len(), 3);
        assert!(buffer.is_full());
        assert_eq!(buffer.get(0), Some(10.0));
        assert_eq!(buffer.get(1), Some(20.0));
        assert_eq!(buffer.get(2), Some(30.0));
        assert_eq!(buffer.get(3), None);
    }

    #[test]
    fn test_circular_overwrite() {
        let mut buffer = CircularBuffer::new(3);
        buffer.push(10.0);
        buffer.push(20.0);
        buffer.push(30.0);
        buffer.push(40.0); // Overwrites 10.0

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get(0), Some(20.0));
        assert_eq!(buffer.get(1), Some(30.0));
        assert_eq!(buffer.get(2), Some(40.0));
    }

    #[test]
    fn test_first_and_last() {
        let mut buffer = CircularBuffer::new(3);
        assert_eq!(buffer.first(), None);
        assert_eq!(buffer.last(), None);

        buffer.push(10.0);
        assert_eq!(buffer.first(), Some(10.0));
        assert_eq!(buffer.last(), Some(10.0));

        buffer.push(20.0);
        buffer.push(30.0);
        assert_eq!(buffer.first(), Some(10.0));
        assert_eq!(buffer.last(), Some(30.0));

        buffer.push(40.0); // Overwrites 10.0
        assert_eq!(buffer.first(), Some(20.0));
        assert_eq!(buffer.last(), Some(40.0));
    }

    #[test]
    fn test_sum() {
        let mut buffer = CircularBuffer::new(3);
        buffer.push(10.0);
        buffer.push(20.0);
        buffer.push(30.0);

        assert_eq!(buffer.sum(), 60.0);

        buffer.push(40.0); // Overwrites 10.0, sum = 20 + 30 + 40 = 90
        assert_eq!(buffer.sum(), 90.0);
    }

    #[test]
    fn test_mean() {
        let mut buffer = CircularBuffer::new(3);
        assert_eq!(buffer.mean(), None);

        buffer.push(10.0);
        buffer.push(20.0);
        buffer.push(30.0);
        assert_eq!(buffer.mean(), Some(20.0));

        buffer.push(40.0); // Overwrites 10.0, mean = (20+30+40)/3 = 30
        assert_eq!(buffer.mean(), Some(30.0));
    }

    #[test]
    fn test_max_min() {
        let mut buffer = CircularBuffer::new(3);
        assert_eq!(buffer.max(), None);
        assert_eq!(buffer.min(), None);

        buffer.push(20.0);
        buffer.push(10.0);
        buffer.push(30.0);

        assert_eq!(buffer.max(), Some(30.0));
        assert_eq!(buffer.min(), Some(10.0));
    }

    #[test]
    fn test_variance_std_dev() {
        let mut buffer = CircularBuffer::new(3);
        buffer.push(2.0);
        buffer.push(4.0);
        buffer.push(6.0);

        // Mean = 4.0
        // Variance = ((2-4)^2 + (4-4)^2 + (6-4)^2) / 3 = (4 + 0 + 4) / 3 = 8/3 ≈ 2.666...
        let variance = buffer.variance().unwrap();
        assert!((variance - 2.666666).abs() < 0.001);

        let std_dev = buffer.std_dev().unwrap();
        assert!((std_dev - 1.632993).abs() < 0.001);
    }

    #[test]
    fn test_clear() {
        let mut buffer = CircularBuffer::new(3);
        buffer.push(10.0);
        buffer.push(20.0);
        buffer.push(30.0);

        assert_eq!(buffer.len(), 3);

        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.first(), None);
        assert_eq!(buffer.last(), None);
    }

    #[test]
    fn test_iterator() {
        let mut buffer = CircularBuffer::new(3);
        buffer.push(10.0);
        buffer.push(20.0);
        buffer.push(30.0);

        let values: Vec<f64> = buffer.iter().copied().collect();
        assert_eq!(values, vec![10.0, 20.0, 30.0]);

        buffer.push(40.0); // Overwrites 10.0
        let values: Vec<f64> = buffer.iter().copied().collect();
        assert_eq!(values, vec![20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_iterator_size_hint() {
        let mut buffer = CircularBuffer::new(5);
        buffer.push(1.0);
        buffer.push(2.0);
        buffer.push(3.0);

        let iter = buffer.iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }
}
