use std::fmt::Write;

/// A cyclic array that can be used to store a fixed number of elements and
/// rotate them in a circular fashion.
///
pub struct CyclicArray<const N: usize, T> {
    data: [T; N],
    start: usize,
}

impl<const N: usize, T> CyclicArray<N, T> {
    /// Create a new cyclic array with the given data.
    pub fn new(data: [T; N]) -> Self {
        Self { data, start: 0 }
    }

    /// Get the array size.
    pub const fn size(&self) -> usize {
        N
    }

    /// Get the current element.
    pub fn get(&self) -> &T {
        &self.data[self.start]
    }

    /// Rotate the array by the given number of steps.
    /// Positive steps rotate the array to the right, negative steps to the left.
    ///
    /// # Arguments
    ///
    /// * `steps` - The number of steps to rotate the array.
    ///
    pub fn rotate(&mut self, steps: usize) {
        self.start = (self.start + steps) % N;
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
    pub fn iter(&self) -> CyclicArrayIterator<N, T> {
        CyclicArrayIterator {
            cyclic_array: self,
            start: self.start,
            index: 0,
        }
    }
}

pub struct CyclicArrayIterator<'a, const N: usize, T> {
    cyclic_array: &'a CyclicArray<N, T>,
    start: usize,
    index: usize,
}

impl<'a, const N: usize, T> Iterator for CyclicArrayIterator<'a, N, T> {
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
    for CyclicArrayIterator<'a, N, T>
{
    fn len(&self) -> usize {
        N
    }
}

impl<'a, const N: usize, T> DoubleEndedIterator
    for CyclicArrayIterator<'a, N, T>
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

impl<const N: usize, T: std::fmt::Debug> std::fmt::Debug for CyclicArray<N, T> {
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
    fn test_cyclic_array() {
        let mut cyclic_array = CyclicArray::new([1, 2, 3, 4, 5]);

        assert_eq!(cyclic_array.get(), &1);
        cyclic_array.rotate(1);
        assert_eq!(cyclic_array.get(), &2);
        cyclic_array.rotate(1);
        assert_eq!(cyclic_array.get(), &3);
        cyclic_array.push(6);
        cyclic_array.rotate(1);
        assert_eq!(cyclic_array.get(), &4);
        cyclic_array.rotate(1);
        assert_eq!(cyclic_array.get(), &5);
        cyclic_array.rotate(2);
        assert_eq!(cyclic_array.get(), &6);
    }

    #[test]
    fn test_cyclic_array_iterator() {
        let cyclic_array = CyclicArray::new([1, 2, 3, 4, 5]);

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
        let cyclic_array = CyclicArray::new([1, 2, 3, 4, 5]);

        let mut iter = cyclic_array.iter();

        assert_eq!(iter.next_back(), Some(&5));
        assert_eq!(iter.next_back(), Some(&4));
        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.next_back(), Some(&2));
        assert_eq!(iter.next_back(), Some(&1));
        assert_eq!(iter.next_back(), None);
    }
}
