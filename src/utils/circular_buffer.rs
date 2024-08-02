/// A cyclic array that can be used to store a fixed number of elements and
/// rotate them in a circular fashion.
///
pub struct CircularBuffer<const N: usize, T> {
    data: [T; N],
    start: usize,
}

impl<const N: usize, T> CircularBuffer<N, T> {
    /// Create a new cyclic array with the given data.
    pub fn new(data: [T; N]) -> Self {
        Self { data, start: 0 }
    }

    /// Rotate the array by the given number of steps.
    /// Positive steps rotate the array to the right, negative steps to the left.
    ///
    /// # Arguments
    ///
    /// * `steps` - The number of steps to rotate the array.
    ///
    pub fn rotate(&mut self, steps: isize) {
        if steps == 0 {
            return;
        } else if steps > 0 {
            self.rotate_right(steps as usize);
        } else {
            self.rotate_left((-steps) as usize);
        }
    }

    fn rotate_left(&mut self, steps: usize) {
        self.start = (self.start + steps) % N;
    }

    fn rotate_right(&mut self, steps: usize) {
        self.start = (self.start + N - steps) % N;
    }

    /// Push a new element to the array.
    ///
    /// # Arguments
    ///
    /// * `element` - The element to push.
    ///
    pub fn push(&mut self, element: T) -> T {
        let end = if self.start == 0 {
            N - 1
        } else {
            self.start - 1
        };

        std::mem::replace(&mut self.data[end], element)
    }

    /// Get an iterator over the elements of the array.
    /// The iterator will start at the current element
    /// and iterate over all elements in the array.
    pub fn iter(&self) -> CircularBufferIterator<N, T> {
        CircularBufferIterator {
            cyclic_array: self,
            start: self.start,
            index: 0,
        }
    }
}

pub struct CircularBufferIterator<'a, const N: usize, T> {
    cyclic_array: &'a CircularBuffer<N, T>,
    start: usize,
    index: usize,
}

impl<'a, const N: usize, T> Iterator for CircularBufferIterator<'a, N, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < N {
            let index = (self.start + self.index) % N;
            self.index += 1;
            Some(&self.cyclic_array.data[index])
        } else {
            None
        }
    }
}

impl<'a, const N: usize, T> ExactSizeIterator
    for CircularBufferIterator<'a, N, T>
{
    fn len(&self) -> usize {
        N
    }
}

impl<'a, const N: usize, T> DoubleEndedIterator
    for CircularBufferIterator<'a, N, T>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < N {
            let index = (self.start + N - self.index - 1) % N;
            self.index += 1;
            Some(&self.cyclic_array.data[index])
        } else {
            None
        }
    }
}

impl<const N: usize, T: std::fmt::Debug> std::fmt::Debug
    for CircularBuffer<N, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        for (i, value) in self.data.iter().enumerate() {
            if i == self.start {
                write!(f, "({:?}), ", value)?;
            } else {
                write!(f, "{:?}, ", value)?;
            }
        }

        write!(f, "]")
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_cyclic_array_iterator() {
        let cyclic_array = CircularBuffer::new([1, 2, 3, 4, 5]);

        let mut iter = cyclic_array.iter();

        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_cyclic_array_iterator_back() {
        let cyclic_array = CircularBuffer::new([1, 2, 3, 4, 5]);

        let mut iter = cyclic_array.iter();

        assert_eq!(iter.next_back(), Some(&5));
        assert_eq!(iter.next_back(), Some(&4));
        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.next_back(), Some(&2));
        assert_eq!(iter.next_back(), Some(&1));
        assert_eq!(iter.next_back(), None);
    }
}
