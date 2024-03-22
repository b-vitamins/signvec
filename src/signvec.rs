use crate::{Sign, Signable};
use fastset::Set;
use nanorand::WyRand;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Bound, Deref, Index, RangeBounds};

const DEFAULT_SET_SIZE: usize = 1000;

/// A vector-like data structure with additional information about the sign of its elements.
///
/// This data structure holds a vector of elements of type `T`, along with sets `pos` and `neg`
/// containing the indices of positive and negative elements respectively. The `SignVec` is used
/// to efficiently store and manipulate elements based on their sign.
///
/// Compared to standard vectors, `SignVec` provides additional functionality for handling
/// elements based on their sign and maintaining sets of positive and negative indices.
///
/// # Type Parameters
///
/// * `T`: The type of elements stored in the `SignVec`, which must implement the `Signable` trait
///        and also be cloneable.
///
/// # Fields
///
/// * `vals`: A vector holding elements of type `T`.
/// * `pos`: A set containing the indices of positive elements in `vals`.
/// * `neg`: A set containing the indices of negative elements in `vals`.
/// * `_marker`: Phantom data field to maintain covariance with the type parameter `T`.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignVec<T>
where
    T: Signable + Clone,
{
    pub vals: Vec<T>,
    pub pos: Set,
    pub neg: Set,
    _marker: PhantomData<T>,
}

impl<T> SignVec<T>
where
    T: Signable + Clone,
{
    /// Appends elements from another vector to the end of this `SignVec`.
    ///
    /// This method appends each element from the provided vector `other` to the end of the `vals`
    /// vector of this `SignVec`. It updates the `pos` and `neg` sets accordingly based on the
    /// sign of each appended element.
    ///
    /// # Arguments
    ///
    /// * `other`: A mutable reference to a vector of elements to be appended.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec, Sign};
    ///
    /// let mut sv = svec![5, -10, 15];
    /// let mut other_vec = vec![25, -30];
    /// let mut other_sv = svec![20, -35];
    ///
    /// sv.append(&mut other_vec);
    /// sv.append(&mut other_sv);
    ///
    /// assert_eq!(sv.len(), 7);
    /// assert_eq!(sv.count(Sign::Plus), 4);
    /// assert_eq!(sv.count(Sign::Minus), 3);
    /// ```
    #[inline(always)]
    pub fn append(&mut self, other: &[T])
    where
        T: Signable + Clone,
    {
        let start_len = self.vals.len();
        other.iter().enumerate().for_each(|(index, e)| {
            let vals_index = start_len + index;
            match e.sign() {
                Sign::Plus => self.pos.insert(vals_index),
                Sign::Minus => self.neg.insert(vals_index),
            };
            self.vals.push(e.clone());
        });
    }
    /// Returns a raw pointer to the underlying data of this `SignVec`.
    ///
    /// This method returns a raw pointer to the first element in the `vals` vector of this `SignVec`.
    ///
    /// # Safety
    ///
    /// The returned pointer is valid for as long as this `SignVec` is not modified or deallocated.
    /// Modifying the `SignVec` or deallocating it invalidates the pointer. It's unsafe to dereference the pointer directly.
    /// However, it can be safely passed to other functions that expect a raw pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let sv = svec![5, -10, 15];
    /// let ptr = sv.as_ptr();
    ///
    /// ```
    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.vals.as_ptr()
    }
    /// Returns a slice containing all elements in this `SignVec`.
    ///
    /// This method returns a slice containing all elements in the `vals` vector of this `SignVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let sv = svec![5, -10, 15];
    /// let slice = sv.as_slice();
    ///
    /// assert_eq!(slice, &[5, -10, 15]);
    /// ```
    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        self.vals.as_slice()
    }

    /// Returns the capacity of the `vals` vector of this `SignVec`.
    ///
    /// This method returns the capacity of the `vals` vector, which is the maximum number of elements
    /// that the vector can hold without reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let sv = svec![5, -10, 15];
    /// assert_eq!(sv.capacity(), 4);
    /// ```
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.vals.capacity()
    }

    /// Clears all elements from this `SignVec`.
    ///
    /// This method removes all elements from the `vals` vector of this `SignVec`, and clears the
    /// `pos` and `neg` sets. The capacity of none of the fields are affected.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sv = svec![5, -10, 15];
    /// sv.clear();
    ///
    /// assert!(sv.is_empty());
    /// ```
    #[inline(always)]
    pub fn clear(&mut self) {
        self.vals.clear();
        self.pos.clear();
        self.neg.clear();
    }
    /// Returns the number of elements with the specified sign in this `SignVec`.
    ///
    /// This method returns the number of elements in the `pos` set if `sign` is `Sign::Plus`, or
    /// the number of elements in the `neg` set if `sign` is `Sign::Minus`.
    ///
    /// # Arguments
    ///
    /// * `sign`: The sign of the elements to count.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, Sign, svec};
    ///
    /// let sv = svec![5, -10, 15];
    ///
    /// assert_eq!(sv.count(Sign::Plus), 2);
    /// assert_eq!(sv.count(Sign::Minus), 1);
    /// ```
    #[inline(always)]
    pub fn count(&self, sign: Sign) -> usize {
        match sign {
            Sign::Plus => self.pos.len(),
            Sign::Minus => self.neg.len(),
        }
    }

    /// Removes consecutive duplicate elements from this `SignVec`.
    ///
    /// This method removes consecutive duplicate elements from the `vals` vector of this `SignVec`.
    /// Elements are considered duplicates if they are equal according to the `PartialEq` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sv = svec![5, 5, -10, 15, 15];
    /// sv.dedup();
    ///
    /// assert_eq!(sv, svec![5, -10, 15]);
    /// ```
    #[inline(always)]
    pub fn dedup(&mut self)
    where
        T: PartialEq + Signable,
    {
        if self.vals.is_empty() {
            return;
        }

        let mut write = 1; // Index to write to.
        for read in 1..self.vals.len() {
            if self.vals[read] != self.vals[read - 1] {
                // Move non-duplicate to the 'write' position if necessary.
                if read != write {
                    self.vals[write] = self.vals[read].clone();
                                                               
                    if self.vals[read].sign() == Sign::Plus {
                        self.pos.remove(&read);
                        self.pos.insert(write);
                    } else {
                        self.neg.remove(&read);
                        self.neg.insert(write);
                    }
                }
                write += 1;
            } else {
                // For duplicates, just remove them from pos and neg sets.
                self.pos.remove(&read);
                self.neg.remove(&read);
            }
        }
        // Truncate the vector to remove excess elements.
        self.vals.truncate(write);
    }

    /// Removes elements from this `SignVec` based on a predicate.
    ///
    /// This method removes elements from the `vals` vector of this `SignVec` based on the provided
    /// predicate `same_bucket`. Elements `x` and `y` are considered duplicates if `same_bucket(&x, &y)`
    /// returns `true`.
    ///
    /// # Arguments
    ///
    /// * `same_bucket`: A predicate used to determine whether two elements are duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, 5, -10, 15, 15];
    /// sign_vec.dedup_by(|x, y| x == y);
    ///
    /// assert_eq!(sign_vec, svec![5, -10, 15]);
    /// ```
    #[inline(always)]
    pub fn dedup_by<F>(&mut self, mut same_bucket: F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        unsafe {
            let mut len = self.vals.len();
            let mut i = 0;
            let vals_ptr = self.vals.as_mut_ptr();
            while i < len {
                let curr = vals_ptr.add(i);
                let mut j = i + 1;
                while j < len {
                    let next = vals_ptr.add(j);
                    if same_bucket(&*curr, &*next) {
                        self.vals.remove(j);
                        self.pos.remove(&j);
                        self.neg.remove(&j);
                        len -= 1;
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }
    }
    /// Removes elements from this `SignVec` based on a key function.
    ///
    /// This method removes elements from the `vals` vector of this `SignVec` based on the key
    /// returned by the provided key function `key`. If the key of two consecutive elements is equal,
    /// the second element is removed.
    ///
    /// # Arguments
    ///
    /// * `key`: A function used to determine the key for each element.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec: SignVec<i32> = svec![5, 6, 10, -10];
    /// sign_vec.dedup_by_key(|x| x.abs());
    ///
    /// assert_eq!(sign_vec, svec![5, 6, 10]);
    /// ```
    #[inline(always)]
    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&T) -> K,
        K: PartialEq,
    {
        unsafe {
            let mut i = 1;
            let vals_ptr = self.vals.as_mut_ptr();
            while i < self.vals.len() {
                // Use while loop to manually control the iteration process, allowing us to adjust 'i' as needed.
                let prev = vals_ptr.add(i - 1);
                let now = vals_ptr.add(i);
                if i > 0 && key(&*prev) == key(&*now) {
                    self.vals.remove(i); // Remove the current item if its key matches the previous item's key.
                                         // Do not increment 'i' so that the next element,
                                         // which shifts into the current index, is compared next.
                    self.pos.remove(&(i));
                    self.neg.remove(&(i));
                } else {
                    i += 1; // Only increment 'i' if no removal was made.
                }
            }
        }
    }

    /// Drains elements from this `SignVec` based on a range.
    ///
    /// This method removes elements from the `vals` vector of this `SignVec` based on the provided
    /// range `range`. It returns a `SignVecDrain` iterator over the removed elements.
    ///
    /// # Arguments
    ///
    /// * `range`: The range of indices to drain elements from.
    ///
    /// # Panics
    ///
    /// Panics if the range is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    /// use std::ops::Bound;
    ///
    /// let mut sign_vec = svec![5, -10, 15, 20];
    /// let drained: Vec<_> = sign_vec.drain(1..3).collect();
    ///
    /// assert_eq!(drained, vec![-10, 15]);
    /// assert_eq!(sign_vec, svec![5, 20]);
    /// ```
    #[inline(always)]
    pub fn drain<R>(&mut self, range: R) -> SignVecDrain<'_, T>
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&e) => e + 1,
            Bound::Excluded(&e) => e,
            Bound::Unbounded => self.vals.len(),
        };

        // Initial validation.
        if start > end || end > self.vals.len() {
            panic!("Drain range out of bounds");
        }

        SignVecDrain {
            sign_vec: self,
            current_index: start,
            drain_end: end,
        }
    }

    /// Extends this `SignVec` with elements from a slice.
    ///
    /// This method appends each element from the provided slice `other` to the end of the `vals`
    /// vector of this `SignVec`. It updates the `pos` and `neg` sets accordingly based on the
    /// sign of each appended element.
    ///
    /// # Arguments
    ///
    /// * `other`: A slice of elements to be appended.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10];
    /// sign_vec.extend_from_slice(&[15, -20]);
    ///
    /// assert_eq!(sign_vec, svec![5, -10, 15, -20]);
    /// ```
    #[inline(always)]
    pub fn extend_from_slice(&mut self, other: &[T]) {
        let offset = self.vals.len();
        self.vals.extend_from_slice(other);
        for (i, e) in other.iter().enumerate() {
            match e.sign() {
                Sign::Plus => self.pos.insert(offset + i),
                Sign::Minus => self.neg.insert(offset + i),
            };
        }
    }

    /// Extends this `SignVec` with elements from within a range.
    ///
    /// This method appends elements from the range `src` within the `vals` vector of this `SignVec`
    /// to the end of the `vals` vector. It updates the `pos` and `neg` sets accordingly based on the
    /// sign of each appended element.
    ///
    /// # Arguments
    ///
    /// * `src`: The range of indices to extend elements from.
    ///
    /// # Panics
    ///
    /// Panics if the range is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    /// use std::ops::Bound;
    ///
    /// let mut sign_vec = svec![5, -10, 15, 20];
    /// sign_vec.extend_from_within(..=2);
    ///
    /// assert_eq!(sign_vec, svec![5, -10, 15, 20, 5, -10, 15]);
    /// ```
    #[inline(always)]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
    {
        let start = match src.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s + 1,
            Bound::Unbounded => 0,
        };
        let end = match src.end_bound() {
            Bound::Included(&e) => e + 1,
            Bound::Excluded(&e) => e,
            Bound::Unbounded => self.vals.len(),
        };
        if start > end || end > self.vals.len() {
            panic!("Invalid range for extend_from_within");
        }

        let offset = self.vals.len();
        self.vals.extend_from_within(start..end);
        for i in start..end {
            match self.vals[i].sign() {
                Sign::Plus => self.pos.insert(offset + i - start),
                Sign::Minus => self.neg.insert(offset + i - start),
            };
        }
    }
    /// Inserts an element at a specified index into this `SignVec`.
    ///
    /// This method inserts the specified `element` at the given `index` into the `vals` vector of
    /// this `SignVec`. It updates the `pos` and `neg` sets accordingly based on the sign of the
    /// inserted element.
    ///
    /// # Arguments
    ///
    /// * `index`: The index at which to insert the element.
    /// * `element`: The element to insert.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10];
    /// sign_vec.insert(1, 15);
    ///
    /// assert_eq!(sign_vec, svec![5, 15, -10]);
    /// ```
    #[inline(always)]
    pub fn insert(&mut self, index: usize, element: T) {
        self.pos = self
            .pos
            .iter()
            .map(|&idx| if idx >= index { idx + 1 } else { idx })
            .collect();
        self.neg = self
            .neg
            .iter()
            .map(|&idx| if idx >= index { idx + 1 } else { idx })
            .collect();
        match element.sign() {
            Sign::Plus => {
                self.pos.insert(index);
            }
            Sign::Minus => {
                self.neg.insert(index);
            }
        };
        self.vals.insert(index, element);
    }

    /// Returns a reference to the set of indices with the specified sign.
    ///
    /// This method returns a reference to the `Set` containing the indices of elements with the
    /// specified `sign` in this `SignVec`.
    ///
    /// # Arguments
    ///
    /// * `sign`: The sign of the elements whose indices are requested.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec, Sign};
    /// use fastset::Set;
    ///
    /// let sign_vec = svec![5, -10, 15, -20];
    ///
    /// assert_eq!(sign_vec.indices(Sign::Plus), &Set::from(&[0, 2]));
    /// assert_eq!(sign_vec.indices(Sign::Minus), &Set::from(&[1, 3]));
    /// ```
    #[inline(always)]
    pub fn indices(&self, sign: Sign) -> &Set {
        match sign {
            Sign::Plus => &self.pos,
            Sign::Minus => &self.neg,
        }
    }

    /// Consumes this `SignVec`, returning a boxed slice of its elements.
    ///
    /// This method consumes the `SignVec`, transforming it into a boxed slice of its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let sign_vec = svec![5, -10];
    /// let boxed_slice = sign_vec.into_boxed_slice();
    ///
    /// assert_eq!(&*boxed_slice, &[5, -10]);
    /// ```
    #[inline(always)]
    pub fn into_boxed_slice(self) -> Box<[T]> {
        self.vals.into_boxed_slice()
    }

    /// Returns `true` if this `SignVec` is empty.
    ///
    /// This method returns `true` if the `vals` vector of this `SignVec` is empty, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let sign_vec: SignVec<i32> = svec![];
    ///
    /// assert!(sign_vec.is_empty());
    /// ```
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.vals.is_empty()
    }

    /// Converts this `SignVec` into a mutable slice without deallocating memory.
    ///
    /// This method consumes the `SignVec` and returns a mutable reference to its elements without
    /// deallocating memory. It is the caller's responsibility to ensure that the memory is properly
    /// deallocated once the reference is no longer needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10];
    /// let slice = sign_vec.leak();
    ///
    /// assert_eq!(slice, &mut [5, -10]);
    /// ```
    #[inline(always)]
    pub fn leak<'a>(self) -> &'a mut [T] {
        let pointer = self.vals.as_ptr();
        let len = self.vals.len();
        std::mem::forget(self);
        unsafe { std::slice::from_raw_parts_mut(pointer as *mut T, len) }
    }

    /// Returns the number of elements in this `SignVec`.
    ///
    /// This method returns the number of elements stored in the `vals` vector of this `SignVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let sign_vec = svec![5, -10, 15];
    ///
    /// assert_eq!(sign_vec.len(), 3);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.vals.len()
    }
    /// Creates a new `SignVec` from a slice of elements.
    ///
    /// This method constructs a new `SignVec` by iterating over the elements in the input slice `input`.
    /// It initializes the `vals` vector with the elements from the slice and populates the `pos` and
    /// `neg` sets based on the sign of each element. The maximum index value is used to initialize the sets.
    ///
    /// # Arguments
    ///
    /// * `input`: A slice containing elements to be stored in the `SignVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec, Sign};
    ///
    /// let mut sign_vec = SignVec::new();
    /// assert_eq!(sign_vec.len(), 0);
    /// assert_eq!(sign_vec.count(Sign::Plus), 0);
    /// assert_eq!(sign_vec.count(Sign::Minus), 0);
    ///
    /// let input_slice = &[5, -10, 15];
    /// sign_vec.extend(input_slice);
    ///
    /// assert_eq!(sign_vec.len(), 3);
    /// assert_eq!(sign_vec.count(Sign::Plus), 2);
    /// assert_eq!(sign_vec.count(Sign::Minus), 1);
    /// ```
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }
    /// Removes and returns the last element from this `SignVec`, or `None` if it is empty.
    ///
    /// This method removes and returns the last element from the `vals` vector of this `SignVec`, if
    /// it exists. It updates the `pos` and `neg` sets accordingly based on the sign of the removed
    /// element.
    ///
    /// # Returns
    ///
    /// * `Some(T)`: The last element from the `vals` vector, if it exists.
    /// * `None`: If the `vals` vector is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10];
    ///
    /// assert_eq!(sign_vec.pop(), Some(-10));
    /// assert_eq!(sign_vec.pop(), Some(5));
    /// assert_eq!(sign_vec.pop(), None);
    /// ```
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        if let Some(topop) = self.vals.pop() {
            let idx = self.vals.len(); // Get the new length after popping.
            match topop.sign() {
                Sign::Plus => {
                    self.pos.remove(&idx);
                }
                Sign::Minus => {
                    self.neg.remove(&idx);
                }
            };
            Some(topop)
        } else {
            None
        }
    }

    /// Appends an element to the end of this `SignVec`.
    ///
    /// This method appends the specified `element` to the end of the `vals` vector of this `SignVec`.
    /// It updates the `pos` and `neg` sets accordingly based on the sign of the appended element.
    ///
    /// # Arguments
    ///
    /// * `element`: The element to append.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10];
    /// sign_vec.push(15);
    ///
    /// assert_eq!(sign_vec, svec![5, -10, 15]);
    /// ```
    #[inline(always)]
    pub fn push(&mut self, element: T) {
        let index = self.vals.len();
        match element.sign() {
            Sign::Plus => self.pos.insert(index),
            Sign::Minus => self.neg.insert(index),
        };
        self.vals.push(element);
    }

    /// Removes and returns the element at the specified index from this `SignVec`.
    ///
    /// This method removes and returns the element at the specified `index` from the `vals` vector of
    /// this `SignVec`. It updates the `pos` and `neg` sets accordingly based on the sign of the
    /// removed element.
    ///
    /// # Arguments
    ///
    /// * `index`: The index of the element to remove.
    ///
    /// # Returns
    ///
    /// * `T`: The element removed from the `vals` vector.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// let removed = sign_vec.remove(1);
    ///
    /// assert_eq!(removed, -10);
    /// assert_eq!(sign_vec, svec![5, 15]);
    /// ```
    #[inline(always)]
    pub fn remove(&mut self, index: usize) -> T {
        self.pos = self
            .pos
            .iter()
            .map(|&idx| if idx > index { idx - 1 } else { idx })
            .collect();
        self.neg = self
            .neg
            .iter()
            .map(|&idx| if idx > index { idx - 1 } else { idx })
            .collect();
        let removed = self.vals.remove(index);
        match removed.sign() {
            Sign::Plus => self.pos.remove(&index),
            Sign::Minus => self.neg.remove(&index),
        };
        removed
    }
    /// Reserves capacity for at least `additional` more elements in `vals`.
    ///
    /// This method reserves capacity for at least `additional` more elements in the `vals` vector of
    /// this `SignVec`. It also reserves capacity in the `pos` and `neg` sets accordingly based on the
    /// new capacity of the `vals` vector.
    ///
    /// # Arguments
    ///
    /// * `additional`: The number of additional elements to reserve capacity for.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10];
    /// sign_vec.reserve(3);
    ///
    /// assert!(sign_vec.capacity() >= 5);
    /// ```
    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        let new_capacity = self.vals.len() + additional;
        self.vals.reserve(additional);
        self.pos.reserve(new_capacity);
        self.neg.reserve(new_capacity);
    }

    /// Reserves the exact capacity for `additional` more elements in `vals`.
    ///
    /// This method reserves the exact capacity for `additional` more elements in the `vals` vector of
    /// this `SignVec`. It also reserves capacity in the `pos` and `neg` sets accordingly based on the
    /// new capacity of the `vals` vector.
    ///
    /// # Arguments
    ///
    /// * `additional`: The exact number of additional elements to reserve capacity for.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10];
    /// sign_vec.reserve_exact(3);
    ///
    /// assert_eq!(sign_vec.capacity(), 5);
    /// ```
    #[inline(always)]
    pub fn reserve_exact(&mut self, additional: usize) {
        let new_capacity = self.vals.len() + additional;
        self.vals.reserve_exact(additional);
        self.pos.reserve(new_capacity);
        self.neg.reserve(new_capacity);
    }

    /// Resizes the `SignVec` in place to a new length.
    ///
    /// This method changes the `len` field of the `vals` vector of this `SignVec`, and adjusts the
    /// elements, `pos`, and `neg` sets accordingly based on the new length and the specified `value`.
    ///
    /// # Arguments
    ///
    /// * `new_len`: The new length of the `SignVec`.
    /// * `value`: The value to initialize new elements with, if `new_len` is greater than the current
    ///            length.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10];
    /// sign_vec.resize(5, 0);
    ///
    /// assert_eq!(sign_vec, svec![5, -10, 0, 0, 0]);
    /// ```
    #[inline(always)]
    pub fn resize(&mut self, new_len: usize, value: T) {
        let old_len = self.vals.len();
        match new_len > old_len {
            true => {
                self.vals.resize(new_len, value.clone());
                match value.sign() {
                    Sign::Plus => (old_len..new_len).for_each(|i| {
                        self.pos.insert(i);
                    }),
                    Sign::Minus => (old_len..new_len).for_each(|i| {
                        self.neg.insert(i);
                    }),
                };
            }
            false => {
                (new_len..old_len).for_each(|i| {
                    self.pos.remove(&i);
                    self.neg.remove(&i);
                });
                self.vals.truncate(new_len);
            }
        }
    }

    /// Resizes the `SignVec` in place to a new length, using a closure to create new values.
    ///
    /// This method changes the `len` field of the `vals` vector of this `SignVec`, and adjusts the
    /// elements, `pos`, and `neg` sets accordingly based on the new length and values generated by the
    /// closure `f`.
    ///
    /// # Arguments
    ///
    /// * `new_len`: The new length of the `SignVec`.
    /// * `f`: A closure that creates new values for elements beyond the current length.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10];
    /// sign_vec.resize_with(5, || 0);
    ///
    /// assert_eq!(sign_vec, svec![5, -10, 0, 0, 0]);
    /// ```
    #[inline(always)]
    pub fn resize_with<F>(&mut self, new_len: usize, mut f: F)
    where
        F: FnMut() -> T,
    {
        let old_len = self.vals.len();
        match new_len > old_len {
            true => {
                (old_len..new_len).for_each(|i| {
                    let value = f();
                    match value.sign() {
                        Sign::Plus => self.pos.insert(i),
                        Sign::Minus => self.neg.insert(i),
                    };
                    self.vals.push(value);
                });
            }
            false => {
                (new_len..old_len).for_each(|i| {
                    self.pos.remove(&i);
                    self.neg.remove(&i);
                });
                self.vals.truncate(new_len);
            }
        }
    }
    /// Retains only the elements specified by the predicate `f`.
    ///
    /// This method retains only the elements specified by the predicate `f` in the `vals` vector of
    /// this `SignVec`. It also adjusts the `pos` and `neg` sets accordingly based on the retained
    /// elements.
    ///
    /// # Arguments
    ///
    /// * `f`: A closure that takes a reference to an element and returns `true` if the element should
    ///        be retained, or `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// sign_vec.retain(|&x| x >= 0);
    ///
    /// assert_eq!(sign_vec, svec![5, 15]);
    /// ```
    #[inline(always)]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.vals.retain(f);
        self.sync();
    }

    /// Retains only the elements specified by the mutable predicate `f`.
    ///
    /// This method retains only the elements specified by the mutable predicate `f` in the `vals`
    /// vector of this `SignVec`. It also adjusts the `pos` and `neg` sets accordingly based on the
    /// retained elements.
    ///
    /// # Arguments
    ///
    /// * `f`: A closure that takes a mutable reference to an element and returns `true` if the
    ///        element should be retained, or `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// sign_vec.retain_mut(|x| *x >= 0);
    ///
    /// assert_eq!(sign_vec, svec![5, 15]);
    /// ```
    #[inline(always)]
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.vals.retain_mut(f);
        self.sync();
    }

    /// Returns a random index of an element with the specified sign.
    ///
    /// This method returns a random index of an element with the specified sign (`Sign::Plus` or
    /// `Sign::Minus`) in the `SignVec`. If no elements with the specified sign exist, `None` is
    /// returned.
    ///
    /// # Arguments
    ///
    /// * `sign`: The sign of the element to search for.
    /// * `rng`: A mutable reference to a random number generator implementing the `WyRand` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, Sign, svec};
    /// use nanorand::WyRand;
    ///
    /// let sign_vec = svec![5, -10, 15];
    /// let mut rng = WyRand::new();
    /// let random_index = sign_vec.random(Sign::Plus, &mut rng);
    ///
    /// assert!(random_index.is_some());
    /// ```
    #[inline(always)]
    pub fn random(&self, sign: Sign, rng: &mut WyRand) -> Option<usize> {
        match sign {
            Sign::Plus => self.pos.random(rng),
            Sign::Minus => self.neg.random(rng),
        }
    }

    /// Returns a random index of an element with a positive sign.
    ///
    /// This method is a specializion of the `random` function, for situations 
    /// where the desired sign (positive, in this case) is known at compile time.
    /// Approximately 25 % faster than calling `random` with `Sign::Plus`
    ///
    /// If no elements with a positive sign exist in the `SignVec`, `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `rng`: A mutable reference to a random number generator implementing the `WyRand` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    /// use nanorand::WyRand;
    ///
    /// let sv = svec![-5, 10, -15];
    /// let mut rng = WyRand::new();
    /// let idx = sv.random_pos(&mut rng).unwrap();
    ///
    /// assert_eq!(sv[idx], 10); // Assumes that `svec!` macro creates a vector where the index of 10 is accessible.
    /// ```
    ///

    #[inline(always)]
    pub fn random_pos(&self, rng: &mut WyRand) -> Option<usize> {
        self.pos.random(rng)
    }

    /// Returns a random index of an element with a positive sign.
    ///
    /// This method is a specializion of the `random` function, for situations 
    /// where the desired sign (negative, in this case) is known at compile time.
    /// Approximately 25 % faster than calling `random` with `Sign::Minus`
    ///
    /// # Arguments
    ///
    /// * `rng`: A mutable reference to a random number generator implementing the `WyRand` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    /// use nanorand::WyRand;
    ///
    /// let sv = svec![5, -10, 15];
    /// let mut rng = WyRand::new();
    /// let idx = sv.random_neg(&mut rng).unwrap();
    ///
    /// assert_eq!(sv[idx], -10);
    /// ```
    #[inline(always)]
    pub fn random_neg(&self, rng: &mut WyRand) -> Option<usize> {
        self.neg.random(rng)
    }

    /// Sets the length of the vector.
    ///
    /// This method sets the length of the vector to `new_len`. If `new_len` is greater than the current
    /// length, additional elements are added and initialized with their default values. If `new_len` is
    /// less than the current length, the excess elements are removed from the vector.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `new_len` is within the bounds of the vector's capacity and that
    /// the elements at indices `old_len..new_len` are properly initialized.
    ///
    /// # Panics
    ///
    /// Panics if `new_len` is greater than the vector's capacity.
    ///
    /// # Examples
    ///
    ///```
    /// use signvec::SignVec;
    ///
    /// unsafe {
    /// let mut sign_vec = SignVec::from(vec![5, -10, 15]);
    ///
    /// sign_vec.resize(5, 0);
    ///
    /// sign_vec.set_len(5);
    ///
    /// assert_eq!(sign_vec.len(), 5);
    /// }
    /// ```
    pub unsafe fn set_len(&mut self, new_len: usize) {
        // Check that new_len is within the bounds of the vector's capacity to avoid out-of-bounds access.
        if new_len > self.vals.capacity() {
            panic!("new_len out of bounds: new_len must be less than or equal to capacity()");
        }

        let old_len = self.vals.len();
        match new_len.cmp(&old_len) {
            std::cmp::Ordering::Greater => {
                // SAFETY: The caller must ensure that the elements at old_len..new_len are properly initialized.
                self.vals.set_len(new_len);
                let vals_ptr = self.vals.as_mut_ptr();
                (old_len..new_len).for_each(|i| {
                    // SAFETY: This dereference is safe under the assumption that elements at old_len..new_len are initialized.
                    match unsafe { &*vals_ptr.add(i) }.sign() {
                        Sign::Plus => {
                            self.pos.insert(i);
                        },
                        Sign::Minus => {
                            self.neg.insert(i);
                        },
                    }
                });
            },
            std::cmp::Ordering::Less => {
                // If the new length is less than the old length, remove indices that are no longer valid.
                (new_len..old_len).for_each(|i| {
                    self.pos.remove(&i);
                    self.neg.remove(&i);
                });
                // SAFETY: This is safe as we're only reducing the vector's length, not accessing any elements.
                self.vals.set_len(new_len);
            },
            std::cmp::Ordering::Equal => {
                // If new_len == old_len, there's no need to do anything.
            },
        }
    }

    /// Sets the value at the specified index.
    ///
    /// This method sets the value at the specified index to the given value. It also updates the
    /// positive (`pos`) and negative (`neg`) sets accordingly based on the sign change of the new
    /// value compared to the old value.
    ///
    /// # Arguments
    ///
    /// * `idx`: The index at which to set the value.
    /// * `val`: The new value to set.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// sign_vec.set(1, 20);
    ///
    /// assert_eq!(sign_vec, svec![5, 20, 15]);
    /// ```
    #[inline(always)]
    pub fn set(&mut self, idx: usize, val: T) {
        if idx >= self.vals.len() {
            panic!(
                "Index out of bounds: index {} length {}",
                idx,
                self.vals.len()
            );
        }
        self.set_unchecked(idx, val);
    }

    /// Sets the value at the specified index without bounds checking.
    ///
    /// This method sets the value at the specified index to the given value without performing any
    /// bounds checking. It also updates the positive (`pos`) and negative (`neg`) sets accordingly
    /// based on the sign change of the new value compared to the old value.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `idx` is within the bounds of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// unsafe {
    ///     sign_vec.set_unchecked(1, 20);
    /// }
    ///
    /// assert_eq!(sign_vec, svec![5, 20, 15]);
    /// ```
    #[inline(always)]
    pub fn set_unchecked(&mut self, idx: usize, mut val: T) {
        let old_val = unsafe { &mut *self.vals.as_mut_ptr().add(idx) };
        let old_sign = old_val.sign();
        let new_sign = val.sign();
        std::mem::swap(old_val, &mut val);
        if old_sign != new_sign {
            match new_sign {
                Sign::Plus => {
                    self.neg.remove(&idx);
                    self.pos.insert(idx);
                }
                Sign::Minus => {
                    self.pos.remove(&idx);
                    self.neg.insert(idx);
                }
            }
        }
    }
    /// Shrinks the capacity of the vector to at least `min_capacity`.
    ///
    /// This method reduces the capacity of the vector to at least `min_capacity` while maintaining
    /// its length. If the current capacity is already less than or equal to `min_capacity`, this
    /// method does nothing.
    ///
    /// # Arguments
    ///
    /// * `min_capacity`: The minimum capacity to which the vector should be shrunk.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let mut sign_vec = SignVec::<f64>::with_capacity(10);
    /// sign_vec.extend(&[5.0, -10.0, 15.0]);
    /// assert!(sign_vec.capacity() >= 10);
    /// sign_vec.shrink_to(2);
    /// assert!(sign_vec.capacity() >= 3);
    ///
    /// ```
    #[inline(always)]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.vals.shrink_to(min_capacity);
        self.pos.shrink_to(min_capacity);
        self.neg.shrink_to(min_capacity);
    }

    /// Shrinks the capacity of the vector to fit its current length.
    ///
    /// This method reduces the capacity of the vector to fit its current length. If the current
    /// capacity is already equal to the length of the vector, this method does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// sign_vec.push(20);
    /// sign_vec.shrink_to_fit();
    ///
    /// assert_eq!(sign_vec.capacity(), 4); // Assuming the default capacity increase strategy
    /// ```
    #[inline(always)]
    pub fn shrink_to_fit(&mut self) {
        self.vals.shrink_to_fit();
        self.pos.shrink_to_fit();
        self.neg.shrink_to_fit();
    }

    /// Returns a mutable slice of the unused capacity of the vector.
    ///
    /// This method returns a mutable slice of the uninitialized memory in the vector's capacity.
    /// Modifying the elements in this slice is safe, but reading from them may lead to undefined
    /// behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    /// use std::mem::MaybeUninit;
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// let unused_capacity = sign_vec.spare_capacity_mut();
    ///
    /// // Fill the spare capacity with new values
    /// for val in unused_capacity.iter_mut() {
    ///     *val = MaybeUninit::new(0);
    /// }
    ///
    /// // Now the spare capacity is filled with zeros
    /// ```
    #[inline(always)]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        let len = self.vals.len();
        let capacity = self.vals.capacity();
        unsafe {
            // SAFETY: The following is safe because MaybeUninit<T> can be uninitialized,
            // and we're only exposing the uninitialized part of the allocated memory.
            std::slice::from_raw_parts_mut(
                self.vals.as_mut_ptr().add(len) as *mut MaybeUninit<T>,
                capacity - len,
            )
        }
    }

    /// Splits the vector into two at the given index.
    ///
    /// This method splits the vector into two at the given index `at`, returning a new vector
    /// containing the elements from index `at` onwards. The original vector will contain the
    /// elements up to but not including `at`. The positive (`pos`) and negative (`neg`) sets
    /// are updated accordingly for both vectors.
    ///
    /// # Arguments
    ///
    /// * `at`: The index at which to split the vector.
    ///
    /// # Panics
    ///
    /// Panics if `at` is greater than the length of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15, -20];
    /// let new_vec = sign_vec.split_off(2);
    ///
    /// assert_eq!(sign_vec, svec![5, -10]);
    /// assert_eq!(new_vec, svec![15, -20]);
    /// ```
    pub fn split_off(&mut self, at: usize) -> SignVec<T> {
        // Ensure 'at' is within bounds to prevent out-of-bounds access.
        if at > self.vals.len() {
            panic!(
                "split_off at index out of bounds: the length is {} but the split index is {}",
                self.vals.len(),
                at
            );
        }
        let new_vals = self.vals.split_off(at);
        let mut new_pos = Set::new(new_vals.len());
        let mut new_neg = Set::new(new_vals.len());
        (0..new_vals.len()).for_each(|i| {
            if self.pos.contains(&(at + i)) {
                self.pos.remove(&(at + i));
                new_pos.insert(i);
            } else if self.neg.remove(&(at + i)) {
                // This also acts as a check, removing the item if present.
                new_neg.insert(i);
            }
        });
        SignVec {
            vals: new_vals,
            pos: new_pos,
            neg: new_neg,
            _marker: PhantomData,
        }
    }
    /// Removes and returns the element at the specified index, replacing it with the last element.
    ///
    /// This method removes and returns the element at the specified `index`, replacing it with the
    /// last element in the vector. The positive (`pos`) and negative (`neg`) sets are updated accordingly.
    ///
    /// # Arguments
    ///
    /// * `index`: The index of the element to be removed.
    ///
    /// # Returns
    ///
    /// The removed element.
    ///
    /// # Panics
    ///
    /// Panics if the `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// let removed = sign_vec.swap_remove(1);
    ///
    /// assert_eq!(removed, -10);
    /// assert_eq!(sign_vec, svec![5, 15]);
    /// ```
    #[inline(always)]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let removed_element = self.vals.swap_remove(index);
        let sign = removed_element.sign();
        match sign {
            Sign::Plus => self.pos.remove(&index),
            Sign::Minus => self.neg.remove(&index),
        };

        if index < self.vals.len() {
            let swapped_element_sign = self.vals[index].sign();
            match swapped_element_sign {
                Sign::Plus => {
                    self.pos.remove(&self.vals.len());
                    self.pos.insert(index);
                }
                Sign::Minus => {
                    self.neg.remove(&self.vals.len());
                    self.neg.insert(index);
                }
            }
        }
        removed_element
    }

    /// Synchronizes the positive and negative sets with the vector's elements.
    ///
    /// This method clears the positive (`pos`) and negative (`neg`) sets, and then re-inserts the
    /// indices of the elements in the vector according to their signs.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// sign_vec.retain(|&x| x >= 0);
    /// sign_vec.sync();
    ///
    /// assert_eq!(sign_vec, svec![5, 15]);
    /// ```
    #[inline(always)]
    pub fn sync(&mut self) {
        self.pos.clear();
        self.neg.clear();
        self.vals.iter().enumerate().for_each(|(idx, val)| {
            match val.sign() {
                Sign::Plus => self.pos.insert(idx),
                Sign::Minus => self.neg.insert(idx),
            };
        });
    }
    /// Truncates the `SignVec` to the specified length.
    ///
    /// This method truncates the `SignVec`, keeping only the first `len` elements. It updates the
    /// positive (`pos`) and negative (`neg`) sets accordingly.
    ///
    /// # Arguments
    ///
    /// * `len`: The new length of the `SignVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// sign_vec.truncate(2);
    ///
    /// assert_eq!(sign_vec, svec![5, -10]);
    /// ```
    #[inline(always)]
    pub fn truncate(&mut self, len: usize) {
        if len < self.vals.len() {
            let old_len = self.vals.len();
            unsafe {
                let vals_ptr = self.vals.as_ptr();
                for i in len..old_len {
                    let val = &*vals_ptr.add(i);
                    match val.sign() {
                        Sign::Plus => self.pos.remove(&i),
                        Sign::Minus => self.neg.remove(&i),
                    };
                }
            }
            self.vals.truncate(len);
        }
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted in the vector.
    ///
    /// This method tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the vector. It updates the capacity of the positive (`pos`) and negative (`neg`) sets
    /// accordingly.
    ///
    /// # Arguments
    ///
    /// * `additional`: The number of additional elements to reserve capacity for.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the capacity was successfully reserved, or an error if the allocation failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// sign_vec.try_reserve(5).unwrap();
    /// ```
    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.vals.try_reserve(additional)?;
        self.pos.reserve(self.vals.len() + additional);
        self.neg.reserve(self.vals.len() + additional);
        Ok(())
    }

    /// Tries to reserve the exact capacity for the vector to hold `additional` more elements.
    ///
    /// This method tries to reserve the exact capacity for the vector to hold `additional` more
    /// elements. It updates the capacity of the positive (`pos`) and negative (`neg`) sets
    /// accordingly.
    ///
    /// # Arguments
    ///
    /// * `additional`: The number of additional elements to reserve capacity for.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the capacity was successfully reserved, or an error if the allocation failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = svec![5, -10, 15];
    /// sign_vec.try_reserve_exact(5).unwrap();
    /// ```
    pub fn try_reserve_exact(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.vals.try_reserve_exact(additional)?;
        self.pos.reserve(self.vals.len() + additional);
        self.neg.reserve(self.vals.len() + additional);
        Ok(())
    }

    /// Returns an iterator over the values with the specified sign.
    ///
    /// This method returns an iterator over the values in the `SignVec` with the specified `sign`.
    ///
    /// # Arguments
    ///
    /// * `sign`: The sign of the values to iterate over.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, Sign, svec};
    ///
    /// let sign_vec = svec![5, -10, 15];
    /// let positive_values: Vec<&i32> = sign_vec.values(Sign::Plus).collect();
    ///
    /// assert_eq!(positive_values, vec![&5, &15]);
    /// ```
    #[inline(always)]
    pub fn values(&self, sign: Sign) -> SignVecValues<T> {
        SignVecValues::new(self, sign)
    }


    /// Creates a new empty `SignVec` with the specified capacity.
    ///
    /// This method creates a new empty `SignVec` with the specified `capacity`.
    ///
    /// # Arguments
    ///
    /// * `capacity`: The capacity of the new `SignVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let sign_vec: SignVec<f64> = SignVec::with_capacity(10);
    /// assert!(sign_vec.is_empty());
    /// ```
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vals: Vec::with_capacity(capacity),
            pos: Set::new(capacity),
            neg: Set::new(capacity),
            _marker: PhantomData,
        }
    }
}

/// An iterator that drains elements from a `SignVec`.
///
/// This iterator yields elements from a `SignVec`, removing them and adjusting the
/// internal positive and negative sets accordingly.
pub struct SignVecDrain<'a, T: 'a + Clone + Signable> {
    /// A mutable reference to the `SignVec` being drained.
    sign_vec: &'a mut SignVec<T>,
    /// The current index being processed during draining.
    current_index: usize,
    /// The end index of the drain operation.
    drain_end: usize,
}

impl<'a, T> Iterator for SignVecDrain<'a, T>
where
    T: Signable + Clone,
{
    type Item = T;

    /// Advances the iterator and returns the next item.
    ///
    /// This method returns `Some(item)` if there are more items to process,
    /// otherwise it returns `None`.
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.drain_end {
            return None;
        }

        // Perform the actual removal.
        let result = self.sign_vec.vals.remove(self.current_index);
        // No need to adjust self.current_index as we always remove the current element.

        // Update pos and neg to reflect the removal.
        // Since we are always removing the current element, we just need to update subsequent indices.
        // Remove the current index from pos or neg if present.
        self.sign_vec.pos.remove(&self.current_index);
        self.sign_vec.neg.remove(&self.current_index);

        // Adjust indices for remaining elements in pos and neg.
        self.sign_vec.pos = self
            .sign_vec
            .pos
            .iter()
            .map(|&i| if i > self.current_index { i - 1 } else { i })
            .collect();
        self.sign_vec.neg = self
            .sign_vec
            .neg
            .iter()
            .map(|&i| if i > self.current_index { i - 1 } else { i })
            .collect();

        // Adjust the drain_end since the vector's length has decreased by one.
        self.drain_end -= 1;

        Some(result)
    }
}

#[derive(Debug)]
pub struct SignVecValues<'a, T> 
where
    T: 'a + Signable + Clone,
{
    // Store a raw pointer to the vector's data.
    vals_ptr: *const T,
    indices_iter: std::slice::Iter<'a, usize>,
}

impl<'a, T> SignVecValues<'a, T> 
where
    T: Signable + Clone,
{
    #[inline(always)]
    pub fn new(sign_vec: &'a SignVec<T>, sign: Sign) -> Self {
        // Obtain a raw pointer to the data of the `vals` vector.
        let vals_ptr = sign_vec.vals.as_ptr();
        let indices_iter = match sign {
            Sign::Plus => (&sign_vec.pos).into_iter(),
            Sign::Minus => (&sign_vec.neg).into_iter(),
        };
        SignVecValues { vals_ptr, indices_iter }
    }
}

impl<'a, T> Iterator for SignVecValues<'a, T> 
where
    T: 'a + Signable + Clone,
{
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.indices_iter.next().map(|&idx| 
            // Safety: The index is assumed to be within bounds, as indices are managed internally
            // and should be valid for the `vals` vector. Accessing the vector's elements
            // via a raw pointer obtained from `as_ptr` assumes that no concurrent modifications
            // occur, and the vector's length does not change during iteration.
            unsafe { &*self.vals_ptr.add(idx) }
        )
    }
}

/// Allows accessing the underlying vector reference of a `SignVec`.
impl<T> AsRef<Vec<T>> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Returns a reference to the underlying vector.
    fn as_ref(&self) -> &Vec<T> {
        &self.vals
    }
}

/// Implements the default trait for `SignVec`.
///
/// This allows creating a new empty `SignVec` with a default capacity.
impl<T> Default for SignVec<T>
where
    T: Signable + Clone,
{
    /// Creates a new empty `SignVec` with a default capacity.
    ///
    /// The default capacity is determined by the `DEFAULT_SET_SIZE` constant.
    fn default() -> Self {
        Self {
            vals: Vec::default(),
            pos: Set::new(DEFAULT_SET_SIZE),
            neg: Set::new(DEFAULT_SET_SIZE),
            _marker: PhantomData,
        }
    }
}

/// Implements extending functionality for references to items.
impl<'a, T> Extend<&'a T> for SignVec<T>
where
    T: Signable + Clone + 'a,
{
    /// Extends the `SignVec` with items from an iterator over references to items.
    ///
    /// This method clones each item from the iterator and appends it to the `SignVec`,
    /// adjusting the positive and negative sets accordingly based on the sign of each item.
    ///
    /// # Arguments
    ///
    /// * `iter`: An iterator over references to items to be extended into the `SignVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, Sign, svec};
    ///
    /// let mut sign_vec: SignVec<i32> = svec![-5, 10, -15];
    /// assert_eq!(sign_vec.len(), 3);
    /// assert_eq!(sign_vec.count(Sign::Plus), 1);
    /// assert_eq!(sign_vec.count(Sign::Minus), 2);
    ///
    /// let items: Vec<i32> = vec![5, -10, 15];
    /// sign_vec.extend(items.iter());
    /// assert_eq!(sign_vec.len(), 6);
    ///
    /// assert_eq!(sign_vec.count(Sign::Plus), 3);
    /// assert_eq!(sign_vec.count(Sign::Minus), 3);
    /// ```
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        for item in iter {
            let index = self.vals.len(); // Get the current length before pushing
            self.vals.push(item.clone()); // Clone the item and push it onto vals
            match item.sign() {
                Sign::Plus => {
                    self.pos.insert(index);
                }
                Sign::Minus => {
                    self.neg.insert(index);
                }
            }
        }
    }
}

/// Implements extending functionality for owned items.
impl<T> Extend<T> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Extends the `SignVec` with items from an iterator over owned items.
    ///
    /// This method appends each item from the iterator to the `SignVec`,
    /// adjusting the positive and negative sets accordingly based on the sign of each item.
    ///
    /// # Arguments
    ///
    /// * `iter`: An iterator over owned items to be extended into the `SignVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{svec, Sign, SignVec};
    ///
    /// let mut sign_vec = svec![-15, 10, -5];
    ///
    /// assert_eq!(sign_vec.len(), 3);
    /// assert_eq!(sign_vec.count(Sign::Plus), 1);
    /// assert_eq!(sign_vec.count(Sign::Minus), 2);
    ///
    /// let items = vec![5, -10, 15];
    /// sign_vec.extend(items.into_iter());
    ///
    /// assert_eq!(sign_vec.len(), 6);
    /// assert_eq!(sign_vec.count(Sign::Plus), 3);
    /// assert_eq!(sign_vec.count(Sign::Minus), 3);
    /// ```
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for item in iter {
            let index = self.vals.len(); // Get the current length before pushing
            match item.sign() {
                Sign::Plus => {
                    self.pos.insert(index);
                }
                Sign::Minus => {
                    self.neg.insert(index);
                }
            }
            self.vals.push(item); // Push the item onto vals
        }
    }
}

impl<T> From<&[T]> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Converts a slice reference into a `SignVec` by cloning its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let slice = &[1, 2, 3, 4, 5];
    /// let sign_vec: SignVec<_> = slice.into();
    /// ```
    fn from(slice: &[T]) -> Self {
        slice.iter().cloned().collect()
    }
}

impl<T, const N: usize> From<&[T; N]> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Converts a shared reference to an array into a `SignVec` by cloning its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let array = &[1, 2, 3, 4, 5];
    /// let sign_vec: SignVec<_> = array.into();
    /// ```
    fn from(array: &[T; N]) -> Self {
        array.iter().cloned().collect()
    }
}

impl<T> From<&mut [T]> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Converts a mutable slice reference into a `SignVec` by cloning its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let mut mutable_slice = &mut [1, -5, 7];
    /// let sign_vec: SignVec<_> = mutable_slice.into();
    /// ```
    fn from(slice: &mut [T]) -> Self {
        slice.iter().cloned().collect()
    }
}

impl<T, const N: usize> From<&mut [T; N]> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Converts a mutable reference to an array into a `SignVec` by cloning its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let mut mutable_array = &mut [1.0, 2.0, 3.0];
    /// let sign_vec: SignVec<_> = mutable_array.into();
    /// ```
    fn from(array: &mut [T; N]) -> Self {
        array.iter().cloned().collect()
    }
}

impl<T> From<&Vec<T>> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Converts a vector reference into a `SignVec` by cloning its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let vec = vec![1, 2, 3, 4, 5];
    /// let sign_vec: SignVec<_> = (&vec).into();
    /// ```
    fn from(vec: &Vec<T>) -> Self {
        vec.iter().cloned().collect()
    }
}

impl<T> From<Vec<T>> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Converts a vector into a `SignVec`, moving its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let vec = vec![1, 2, 3, 4, 5];
    /// let sign_vec: SignVec<_> = vec.into();
    /// ```
    fn from(vec: Vec<T>) -> Self {
        vec.into_iter().collect()
    }
}

impl<T> From<SignVec<T>> for Vec<T>
where
    T: Signable + Clone,
{
    /// Converts a `SignVec` into a `Vec`, moving its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let sv = svec![1, 2, 3, 4, 5];
    /// let vec: Vec<_> = Vec::from(&sv);
    /// ```
    fn from(sign_vec: SignVec<T>) -> Self {
        sign_vec.vals.clone()
    }
}

impl<T> From<&SignVec<T>> for Vec<T>
where
    T: Signable + Clone,
{
    /// Converts a reference to `SignVec` into a `Vec`, cloning its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let sign_vec: SignVec<i32> = svec![1, 2, 3, 4, 5];
    /// let vec: Vec<i32> = Vec::from(&sign_vec);
    ///
    /// ```
    fn from(sign_vec: &SignVec<T>) -> Self {
        sign_vec.vals.clone()
    }
}

impl<T, const N: usize> From<[T; N]> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Converts an owned array into a `SignVec`, moving its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let array = [1, 2, 3, 4, 5];
    /// let sign_vec: SignVec<_> = array.into();
    /// ```
    fn from(array: [T; N]) -> Self {
        array.into_iter().collect()
    }
}

impl<T> FromIterator<T> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Constructs a `SignVec` from an iterator, cloning each element.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let iter = vec![1, -2, 3, -4, 5].into_iter();
    /// let sign_vec: SignVec<_> = iter.collect();
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Vec::new();
        let mut pos = Set::new(DEFAULT_SET_SIZE);
        let mut neg = Set::new(DEFAULT_SET_SIZE);

        for (i, item) in iter.into_iter().enumerate() {
            if item.sign() == Sign::Plus {
                pos.insert(i);
            } else {
                neg.insert(i);
            }
            vec.push(item);
        }

        SignVec {
            vals: vec,
            pos,
            neg,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> FromIterator<&'a T> for SignVec<T>
where
    T: 'a + Signable + Clone,
{
    /// Constructs a `SignVec` from an iterator of references, cloning each element.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let iter = vec![&1, &-2, &3, &-4, &5].into_iter();
    /// let sign_vec: SignVec<_> = iter.collect();
    /// ```
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        let mut vec = Vec::new();
        let mut pos = Set::new(DEFAULT_SET_SIZE);
        let mut neg = Set::new(DEFAULT_SET_SIZE);

        for (i, item) in iter.into_iter().enumerate() {
            let cloned_item = item.clone();
            if cloned_item.sign() == Sign::Plus {
                pos.insert(i);
            } else {
                neg.insert(i);
            }
            vec.push(cloned_item);
        }

        SignVec {
            vals: vec,
            pos,
            neg,
            _marker: PhantomData,
        }
    }
}

impl<T> IntoIterator for SignVec<T>
where
    T: Signable + Clone,
{
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;

    /// Converts the `SignVec` into an iterator consuming the original `SignVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let sign_vec = SignVec::from(vec![1, -2, 3]);
    /// let mut iter = sign_vec.into_iter();
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(-2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.vals.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a SignVec<T>
where
    T: Signable + Clone,
{
    type Item = &'a T;
    type IntoIter = ::std::slice::Iter<'a, T>;

    /// Converts the `SignVec` into an iterator yielding references to elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let sign_vec = SignVec::from(vec![1, -2, 3]);
    /// let mut iter = sign_vec.iter();
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&-2));
    /// assert_eq!(iter.next(), Some(&3));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.vals.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SignVec<T>
where
    T: Signable + Clone,
{
    type Item = &'a mut T;
    type IntoIter = ::std::slice::IterMut<'a, T>;

    /// Converts the `SignVec` into an iterator yielding mutable references to elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::{SignVec, svec};
    ///
    /// let mut sign_vec = SignVec::from(vec![1, -2, 3]);
    /// for elem in &mut sign_vec {
    ///    *elem += 1;
    /// }
    /// assert_eq!(sign_vec, svec![2, -1, 4]);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.vals.iter_mut()
    }
}

// Allowing indexing into SignVec to get a reference to an element
impl<T> Index<usize> for SignVec<T>
where
    T: Signable + Clone,
{
    type Output = T;

    /// Returns a reference to the element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let sign_vec = SignVec::from(vec![1, -2, 3]);
    /// assert_eq!(sign_vec[0], 1);
    /// assert_eq!(sign_vec[1], -2);
    /// assert_eq!(sign_vec[2], 3);
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        &self.vals[index]
    }
}

// Allowing dereferencing to a slice of SignVec values
impl<T> Deref for SignVec<T>
where
    T: Signable + Clone,
{
    type Target = [T];

    /// Dereferences the SignVec to a slice of its values.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let sign_vec = SignVec::from(vec![1, -2, 3]);
    /// assert_eq!(&*sign_vec, &[1, -2, 3]);
    /// ```
    fn deref(&self) -> &Self::Target {
        &self.vals
    }
}

impl<T> Borrow<[T]> for SignVec<T>
where
    T: Signable + Clone,
{
    /// Borrows the SignVec as a slice of its values.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    /// use std::borrow::Borrow;
    ///
    /// let sign_vec = SignVec::<i32>::from(vec![1, -2, 3]);
    /// assert_eq!(<SignVec<i32> as std::borrow::Borrow<[i32]>>::borrow(&sign_vec), &[1, -2, 3]);
    /// ```
    fn borrow(&self) -> &[T] {
        &self.vals
    }
}

impl<T> Hash for SignVec<T>
where
    T: Signable + Clone + Hash,
{
    /// Computes the hash value for the SignVec.
    ///
    /// The hash value includes the length of the SignVec and the hash value of each element.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    /// use std::hash::{Hash, Hasher};
    ///
    /// let mut hasher = std::collections::hash_map::DefaultHasher::new();
    /// let sign_vec = SignVec::from(vec![1, -2, 3]);
    /// sign_vec.hash(&mut hasher);
    /// let hash_value = hasher.finish();
    /// ```
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vals.len().hash(state);
        for item in &self.vals {
            item.hash(state);
        }
    }
}

impl<T> Ord for SignVec<T>
where
    T: Signable + Clone + Ord,
{
    /// Compares two SignVecs lexicographically.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let vec1 = SignVec::from(vec![1, -2, 3]);
    /// let vec2 = SignVec::from(vec![1, -3, 4]);
    ///
    /// assert!(vec1 > vec2);
    /// ```
    fn cmp(&self, other: &Self) -> Ordering {
        self.vals.cmp(&other.vals)
    }
}

impl<T> PartialOrd for SignVec<T>
where
    T: Signable + Clone + PartialOrd,
{
    /// Compares two SignVecs lexicographically.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let vec1 = SignVec::from(vec![1, -2, 3]);
    /// let vec2 = SignVec::from(vec![1, -3, 4]);
    ///
    /// assert!(vec1 > vec2);
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.vals.partial_cmp(&other.vals)
    }
}

impl<T> Eq for SignVec<T> where T: Signable + Clone + Eq {}

impl<T> PartialEq for SignVec<T>
where
    T: Signable + Clone + PartialEq,
{
    /// Checks if two SignVecs are equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let vec1 = SignVec::from(vec![1, -2, 3]);
    /// let vec2 = SignVec::from(vec![1, -2, 3]);
    ///
    /// assert_eq!(vec1, vec2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.vals.eq(&other.vals)
    }
}

// Implementations for comparisons between SignVec and slices, mutable slices, arrays, and vectors

impl<T, U> PartialEq<&[U]> for SignVec<T>
where
    T: PartialEq<U> + Signable + Clone,
{
    /// Checks if a SignVec is equal to a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let vec = SignVec::<i32>::from(vec![1, -2, 3]);
    ///
    /// assert_eq!(vec, &[1, -2, 3] as &[i32]);
    /// ```
    fn eq(&self, other: &&[U]) -> bool {
        self.vals.eq(other)
    }
}

impl<T, U> PartialEq<&mut [U]> for SignVec<T>
where
    T: PartialEq<U> + Signable + Clone,
{
    /// Checks if a SignVec is equal to a mutable slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let vec = SignVec::<f64>::from(vec![1.0, -2.0, 3.0]);
    ///
    /// assert_eq!(vec, &[1.0, -2.0, 3.0] as &[f64]);
    /// ```

    fn eq(&self, other: &&mut [U]) -> bool {
        self.vals.eq(*other)
    }
}

impl<T, U, const N: usize> PartialEq<[U; N]> for SignVec<T>
where
    T: PartialEq<U> + Signable + Clone,
{
    /// Checks if a SignVec is equal to an array.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let vec = SignVec::from(vec![1, -2, 3]);
    ///
    /// assert_eq!(vec, [1, -2, 3]);
    /// ```
    fn eq(&self, other: &[U; N]) -> bool {
        self.vals.eq(other)
    }
}

impl<T, U> PartialEq<Vec<U>> for SignVec<T>
where
    T: PartialEq<U> + Signable + Clone,
{
    /// Checks if a SignVec is equal to a vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use signvec::SignVec;
    ///
    /// let vec = SignVec::from(vec![1, -2, 3]);
    ///
    /// assert_eq!(vec, vec![1, -2, 3]);
    /// ```
    fn eq(&self, other: &Vec<U>) -> bool {
        self.vals.eq(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svec;
    use fastset::set;
    use std::collections::HashSet;

    #[derive(Clone, Eq, PartialEq, Default)]
    struct Account {
        balance: i32, // Assuming balance can be positive or negative
    }

    impl Account {
        fn new(balance: i32) -> Self {
            Account { balance }
        }

        fn balance(&self) -> i32 {
            self.balance
        }
    }

    impl Signable for Account {
        fn sign(&self) -> Sign {
            if self.balance >= 0 {
                Sign::Plus
            } else {
                Sign::Minus
            }
        }
    }

    #[test]
    fn test_append() {
        // Test appending positive elements
        let mut vec = svec![-1];
        let other = vec![2, -3];
        vec.append(&other);
        assert_eq!(vec.as_slice(), &[-1, 2, -3]);
        assert_eq!(vec.count(Sign::Plus), 1);
        assert_eq!(vec.count(Sign::Minus), 2);

        // Test appending negative elements
        let mut vec = svec![];
        let other = vec![-2, 3];
        vec.append(&other);
        assert_eq!(vec.as_slice(), &[-2, 3]);
        assert_eq!(vec.count(Sign::Plus), 1);
        assert_eq!(vec.count(Sign::Minus), 1);
    }

    #[test]
    fn test_as_ptr() {
        let vec: SignVec<i32> = svec![1, -2, 3];
        let ptr = vec.as_ptr();
        unsafe {
            assert_eq!(*ptr, 1); // First element
            assert_eq!(*ptr.offset(1), -2); // Second element
            assert_eq!(*ptr.offset(2), 3); // Third element
        }
    }
    #[test]
    fn test_as_slice() {
        let vec = svec![1, -2, 3];
        let slice = vec.as_slice();
        assert_eq!(slice, &[1, -2, 3]);
    }

    #[test]
    fn test_capacity() {
        let vec = SignVec::<i32>::with_capacity(10);
        assert_eq!(vec.capacity(), 10);
    }

    #[test]
    fn test_clear() {
        let mut vec = svec![1, -2, 3];
        vec.clear();
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 4);
    }
    #[test]
    fn test_count() {
        let mut vec = svec![1, -2, 3, -4];
        assert_eq!(vec.count(Sign::Plus), 2);
        assert_eq!(vec.count(Sign::Minus), 2);

        vec.push(5);
        assert_eq!(vec.count(Sign::Plus), 3);
        assert_eq!(vec.count(Sign::Minus), 2);
    }

    #[test]
    fn test_dedup() {
        // Test deduplication of positive elements
        let mut vec = svec![1, 2, 2, -3, -3, 4];
        vec.dedup();
        assert_eq!(vec.as_slice(), &[1, 2, -3, 4]);

        // Test deduplication of negative elements
        let mut vec = svec![1, 2, -2, -3, -3, 4];
        vec.dedup();
        assert_eq!(vec.as_slice(), &[1, 2, -2, -3, 4]);
    }

    #[test]
    fn test_dedup_by() {
        // Test deduplication using a custom equality function
        let mut vec = svec![10, -5, 10, -5];
        vec.dedup_by(|a, b| a == b);
        assert_eq!(vec.as_slice(), &[10, -5]);

        // Test deduplication of complex objects based on a specific property
        let mut vec = svec![
            Account::new(100),
            Account::new(-50),
            Account::new(100),
            Account::new(-50),
        ];
        vec.dedup_by(|a, b| a.balance() == b.balance());
        assert_eq!(vec.as_slice().len(), 2);
    }

    #[test]
    fn test_dedup_by_key() {
        // Test deduplication based on a derived property
        let mut vec = svec![
            Account::new(100),
            Account::new(100),
            Account::new(-50),
            Account::new(-50),
        ];
        vec.dedup_by_key(|a| a.balance());
        assert_eq!(vec.as_slice().len(), 2);
    }

    #[test]
    fn test_drain() {
        // Test draining a range from the middle
        let mut vec = svec![1, 2, 3, 4, 5];
        let drained_elements: Vec<_> = vec.drain(1..4).collect();
        assert_eq!(drained_elements, vec![2, 3, 4]);
        assert_eq!(vec.as_slice(), &[1, 5]);

        // Test draining the entire vector
        let mut vec = svec![1, 2, 3, 4, 5];
        let drained_elements: Vec<_> = vec.drain(..).collect();
        assert_eq!(drained_elements, vec![1, 2, 3, 4, 5]);
        assert!(vec.is_empty());

        // Test draining an empty range
        let mut vec = svec![1, 2, 3, 4, 5];
        let drained_elements: Vec<_> = vec.drain(5..5).collect();
        assert!(drained_elements.is_empty());
        assert_eq!(vec.as_slice(), &[1, 2, 3, 4, 5]);

        // Test draining with excluded end bound
        let mut vec = svec![1, 2, 3, 4, 5];
        let drained_elements: Vec<_> = vec.drain(..=2).collect();
        assert_eq!(drained_elements, vec![1, 2, 3]);
        assert_eq!(vec.as_slice(), &[4, 5]);
    }
    #[test]
    fn test_extend_from_slice() {
        let mut vec = svec![];
        let other = vec![1, 2, 3];
        vec.extend_from_slice(&other);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_extend_from_within() {
        let mut vec = svec![0, 1, 2, 3, 4];
        vec.extend_from_within(2..);
        assert_eq!(vec.as_slice(), &[0, 1, 2, 3, 4, 2, 3, 4]);
        vec.extend_from_within(5..);
        assert_eq!(vec.as_slice(), &[0, 1, 2, 3, 4, 2, 3, 4, 2, 3, 4]);
    }
    #[test]
    fn test_insert() {
        let mut vec = svec![1, 2, 3];
        vec.insert(1, 4);
        assert_eq!(vec.as_slice(), &[1, 4, 2, 3]);
        assert_eq!(vec.count(Sign::Plus), 4);
    }

    #[test]
    fn test_indices() {
        let vec = svec![1, -2, 3, -4, 5];
        assert_eq!(vec.indices(Sign::Plus), &Set::from(vec![0, 2, 4]));
        assert_eq!(vec.indices(Sign::Minus), &Set::from(vec![1, 3]));
    }

    #[test]
    fn test_into_boxed_slice() {
        let vec = svec![1, 2, 3];
        let boxed_slice: Box<[i32]> = vec.into_boxed_slice();
        assert_eq!(&*boxed_slice, &[1, 2, 3]);
    }

    #[test]
    fn test_is_empty() {
        let vec = SignVec::<i32>::new();
        assert!(vec.is_empty());
    }

    #[test]
    fn test_leak() {
        let vec = svec![1, 2, 3];
        let leaked_slice = vec.leak();
        assert_eq!(leaked_slice, &[1, 2, 3]);
    }

    #[test]
    fn test_len() {
        let vec = svec![1, 2, 3];
        assert_eq!(vec.len(), 3);
    }
    #[test]
    fn test_new() {
        let mut sv: SignVec<i32> = SignVec::new();
        let input = vec![1, -2, 3, -4, 5];
        input.iter().for_each(|&i| {
            sv.push(i);
        });
        assert_eq!(sv.vals, input);
        assert_eq!(sv.as_slice(), &[1, -2, 3, -4, 5]);
        assert_eq!(sv.count(Sign::Plus), 3);
        assert_eq!(sv.count(Sign::Minus), 2);
    }

    #[test]
    fn test_pop() {
        let mut vec = svec![1, 2, -3, 4];
        assert_eq!(vec.len(), 4);
        assert!(vec.indices(Sign::Plus).contains(&3));
        assert_eq!(vec.pop(), Some(4));
        assert_eq!(vec.len(), 3);
        assert!(!vec.indices(Sign::Plus).contains(&3));
        assert_eq!(vec.indices(Sign::Plus), &set![0, 1]);
        assert_eq!(vec.indices(Sign::Minus), &set![2]);
        assert_eq!(vec.pop(), Some(-3));
        assert_eq!(vec.len(), 2);
        assert!(!vec.indices(Sign::Minus).contains(&2));
        assert_eq!(vec.indices(Sign::Plus), &set![0, 1]);
        assert_eq!(vec.indices(Sign::Minus), &set![]);
    }
    #[test]
    fn test_push() {
        let mut vec = svec![];
        vec.push(1);
        vec.push(-2);
        vec.push(3);
        assert_eq!(vec.as_slice(), &[1, -2, 3]);
        assert_eq!(vec.count(Sign::Plus), 2);
        assert_eq!(vec.count(Sign::Minus), 1);
    }

    #[test]
    fn test_remove() {
        let mut vec = svec![1, -2, 3];
        vec.remove(1);
        assert_eq!(vec.as_slice(), &[1, 3]);
        assert_eq!(vec.count(Sign::Plus), 2);
        assert_eq!(vec.count(Sign::Minus), 0);
    }
    #[test]
    fn test_reserve() {
        let mut vec = svec![1, -2, 3];
        vec.reserve(5);
        assert!(vec.capacity() >= 5);
    }

    #[test]
    fn test_reserve_exact() {
        let mut vec = svec![1, -2, 3];
        vec.reserve_exact(5);
        assert!(vec.capacity() >= 8);
    }

    #[test]
    fn test_resize() {
        let mut vec = svec![1, -2, 3];
        vec.resize(5, 0);
        assert_eq!(vec.as_slice(), &[1, -2, 3, 0, 0]);
        assert_eq!(vec.count(Sign::Plus), 4);
        assert_eq!(vec.count(Sign::Minus), 1);
    }

    #[test]
    fn test_resize_with() {
        let mut vec = svec![1, -2, 3];
        vec.resize_with(5, || -1);
        assert_eq!(vec.as_slice(), &[1, -2, 3, -1, -1]);
        assert_eq!(vec.count(Sign::Plus), 2);
        assert_eq!(vec.count(Sign::Minus), 3);
    }

    #[test]
    fn test_retain() {
        let mut vec = svec![1, -2, 3];
        vec.retain(|&x| x > 0);
        assert_eq!(vec.as_slice(), &[1, 3]);
        assert_eq!(vec.count(Sign::Plus), 2);
        assert_eq!(vec.count(Sign::Minus), 0);
    }

    #[test]
    fn test_retain_mut() {
        let mut vec = svec![1, -2, 3];
        vec.retain_mut(|x| *x > 0);
        assert_eq!(vec.as_slice(), &[1, 3]);
        assert_eq!(vec.count(Sign::Plus), 2);
        assert_eq!(vec.count(Sign::Minus), 0);
    }

    #[test]
    fn test_random() {
        let mut svec = svec![1, -1, 2, -2, 3];
        let mut rng = WyRand::new();
        let mut observed_values = HashSet::new();
        for _ in 0..100 {
            if let Some(value) = svec.random(Sign::Plus, &mut rng) {
                assert!(
                    svec.pos.contains(&value),
                    "Randomly selected value should be in the set"
                );
                observed_values.insert(value);
            }
        }
        // Check that multiple distinct values are observed
        assert!(
            observed_values.len() > 1,
            "Random should return different values over multiple calls"
        );
        // Test with empty set
        svec.clear();
        assert!(
            svec.random(Sign::Minus, &mut rng).is_none(),
            "Random should return None for an empty set"
        );
        assert!(
            svec.random(Sign::Plus, &mut rng).is_none(),
            "Random should return None for an empty set"
        );
    }

    #[test]
    fn test_set_len() {
        let mut vec = svec![1, -2, 3];
        vec.resize(5, 0); // Reserve extra capacity
        unsafe {
            vec.set_len(5);
        }
        assert_eq!(vec.as_slice(), &[1, -2, 3, 0, 0]);
        assert_eq!(vec.count(Sign::Plus), 4);
        assert_eq!(vec.count(Sign::Minus), 1);

        unsafe {
            vec.set_len(2);
        }
        assert_eq!(vec.as_slice(), &[1, -2]);
        assert_eq!(vec.count(Sign::Plus), 1);
        assert_eq!(vec.count(Sign::Minus), 1);
    }

    #[test]
    #[should_panic(expected = "new_len out of bounds")]
    fn test_set_len_out_of_bounds() {
        let mut vec = svec![1, -2, 3];
        unsafe {
            vec.set_len(10);
        } // Attempt to set length beyond capacity, should panic
    }

    #[test]
    fn test_set_unchecked() {
        let mut vec = svec![1, -2, 3];
        assert_eq!(vec.count(Sign::Plus), 2);
        assert_eq!(vec.count(Sign::Minus), 1);
        vec.set_unchecked(1, 5); // Change -2 to 5
        assert_eq!(vec.as_slice(), &[1, 5, 3]);
        assert_eq!(vec.count(Sign::Plus), 3);
        assert_eq!(vec.count(Sign::Minus), 0);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_set_out_of_bounds() {
        let mut vec = svec![1, -2, 3];
        vec.set(10, 5); // Attempt to set element at index 10, should panic
    }

    #[test]
    fn test_shrink_to() {
        let mut vec = SignVec::<i32>::with_capacity(10);
        vec.extend_from_slice(&[1, -2, 3]);
        assert!(vec.capacity() >= 10);
        vec.shrink_to(5);
        assert!(vec.capacity() >= 5);
        vec.shrink_to(0);
        assert!(vec.capacity() >= 3);
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut vec = svec![1, -2, 3];
        vec.reserve(10); // Reserve extra capacity
        vec.shrink_to_fit();
        assert_eq!(vec.capacity(), 3);
    }

    #[test]
    fn test_spare_capacity_mut() {
        let mut vec = svec![1, -2, 3];
        vec.reserve(10); // Reserve extra capacity
        let expected_spare_capacity = vec.capacity() - vec.len();
        let spare_capacity = vec.spare_capacity_mut();
        assert_eq!(spare_capacity.len(), expected_spare_capacity); // Adjusted expectation
    }

    #[test]
    fn test_split_off() {
        let mut vec = svec![1, -2, 3];
        vec.push(4);
        let new_vec = vec.split_off(2);
        assert_eq!(vec.len(), 2);
        assert_eq!(new_vec.len(), 2);
        assert_eq!(vec.as_slice(), &[1, -2]);
        assert_eq!(new_vec.as_slice(), &[3, 4]);
    }

    #[test]
    fn test_swap_remove() {
        let mut vec = svec![1, -2, 3];
        let removed = vec.swap_remove(1);
        assert_eq!(removed, -2);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.as_slice(), &[1, 3]);
    }

    #[test]
    fn test_sync() {
        let mut vec = svec![1, -2, 3];
        vec.remove(1); // Remove an element to make the sync necessary
        vec.sync();
        assert_eq!(vec.count(Sign::Plus), 2);
        assert_eq!(vec.count(Sign::Minus), 0);

        let mut vec2 = svec![1, -1, 2];
        vec2.vals[0] = -3; // Manually change a value to test sync
        vec2.sync();
        assert!(vec2.indices(Sign::Plus).contains(&2));
        assert!(vec2.indices(Sign::Minus).contains(&0));
    }

    #[test]
    fn test_truncate() {
        let mut vec = svec![1, -2, 3];
        vec.truncate(2);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.as_slice(), &[1, -2]);
    }

    #[test]
    fn test_try_reserve() {
        let mut vec = svec![1, -2, 3];
        vec.try_reserve(2).unwrap();
        assert!(vec.capacity() >= 5); // Original capacity + 2

        // Try reserving exact additional capacity
        vec.try_reserve_exact(3).unwrap();
        assert!(vec.capacity() >= 6); // Original capacity + 3
    }

    #[test]
    fn test_values() {
        let vec = svec![1, -2, 3];
        assert_eq!(vec.values(Sign::Plus).collect::<Vec<_>>(), vec![&1, &3]);
        assert_eq!(vec.values(Sign::Minus).collect::<Vec<_>>(), vec![&-2]);
    }

    #[test]
    fn test_with_capacity() {
        let vec = SignVec::<i32>::with_capacity(5);
        assert_eq!(vec.capacity(), 5);
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_set_and_set_unchecked() {
        let mut sign_vec = svec![1, -1, 2];
        sign_vec.set(1, 3); // Changing -1 to 3
        assert_eq!(sign_vec.vals[1], 3);
        // Assertions to ensure `pos` and `neg` are correctly updated
    }

    #[test]
    fn test_len_and_is_empty() {
        let vec = SignVec::<i32>::new();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());

        let vec = svec![1, -1];
        assert_eq!(vec.len(), 2);
        assert!(!vec.is_empty());
    }

    #[test]
    fn test_indices_values_count() {
        let vec = svec![1, -1, 2, -2, 3];
        assert_eq!(vec.count(Sign::Plus), 3);
        assert_eq!(vec.count(Sign::Minus), 2);

        let plus_indices: Vec<usize> = vec.indices(Sign::Plus).iter().copied().collect();
        assert_eq!(plus_indices, vec![0, 2, 4]);

        let minus_values: Vec<&i32> = vec.values(Sign::Minus).collect();
        assert_eq!(minus_values, vec![&-1, &-2]);
    }

    #[test]
    fn test_capacity_and_with_capacity() {
        let vec = SignVec::<isize>::with_capacity(10);
        assert!(vec.capacity() >= 10);
    }

    #[test]
    fn test_reserve_and_reserve_exact() {
        let mut vec = svec![1, -1, 2];
        vec.reserve(8);
        assert!(vec.capacity() >= 10);
        vec.reserve_exact(20);
        assert!(vec.capacity() >= 20);
    }
    #[test]
    fn test_shrink_to_fit_and_shrink_to() {
        let mut vec = SignVec::with_capacity(10);
        vec.push(1);
        vec.push(-1);
        vec.shrink_to_fit();
        assert_eq!(vec.capacity(), 2);

        vec.shrink_to(10);
        assert!(vec.capacity() >= 2);
    }
    #[test]
    fn test_into_boxed_slice_and_truncate() {
        let mut vec = svec![1, -1, 2, -2, 3];
        vec.truncate(3);
        assert_eq!(vec.len(), 3);

        let boxed_slice = vec.into_boxed_slice();
        assert_eq!(boxed_slice.len(), 3);
    }
    #[test]
    fn test_as_slice_and_as_ptr() {
        let vec = svec![1, -1, 2];
        let slice = vec.as_slice();
        assert_eq!(slice, &[1, -1, 2]);

        let ptr = vec.as_ptr();
        unsafe {
            assert_eq!(*ptr, 1);
        }
    }
    #[test]
    fn test_swap_remove_and_remove() {
        let mut vec = svec![1, -1, 2];
        assert_eq!(vec.swap_remove(1), -1);
        assert_eq!(vec.len(), 2);

        assert_eq!(vec.remove(0), 1);
        assert_eq!(vec.len(), 1);
    }
    #[test]
    fn test_push_and_append() {
        let mut vec = svec![1, 2];
        vec.push(-1);
        assert_eq!(vec.len(), 3);
        assert!(vec.indices(Sign::Minus).contains(&2));

        let other = vec![3, -2];
        vec.append(&other);
        assert_eq!(vec.len(), 5);
        assert!(vec.indices(Sign::Plus).contains(&3));
        assert!(vec.indices(Sign::Minus).contains(&4));
    }
    #[test]
    fn test_insert_and_retain() {
        let mut vec = svec![1, 2, 3, 2];
        vec.insert(1, -1);
        assert_eq!(vec.len(), 5);
        assert_eq!(*vec.values(Sign::Minus).next().unwrap(), -1);
        assert_eq!(vec.indices(Sign::Plus), &set![0, 2, 3, 4]);
        assert_eq!(vec.indices(Sign::Minus), &set![1]);
        vec.retain(|&x| x != -1);
        assert_eq!(vec.len(), 4);
        assert!(!vec.indices(Sign::Minus).contains(&1));
        assert_eq!(vec.indices(Sign::Plus), &set![0, 1, 2, 3]);
        assert_eq!(vec.indices(Sign::Minus), &set![]);
    }

    // Helper function to create a SignVec and verify its contents
    fn check_signvec_contents(vec: SignVec<i32>, expected_vals: &[i32]) {
        assert_eq!(vec.vals, expected_vals);
    }

    #[test]
    fn from_slice() {
        let slice: &[i32] = &[1, -2, 3];
        let sign_vec = SignVec::from(slice);
        check_signvec_contents(sign_vec, slice);
    }

    #[test]
    fn from_array() {
        let array: &[i32; 3] = &[1, -2, 3];
        let sign_vec = SignVec::from(array);
        check_signvec_contents(sign_vec, array);
    }

    #[test]
    fn from_owned_array() {
        let array = [1, -2, 3]; // Owned array
        let sign_vec = SignVec::from(array);
        check_signvec_contents(sign_vec, &array);
    }

    // Tests for AsRef<Vec<T>> for SignVec<T>
    #[test]
    fn test_as_ref() {
        let sign_vec = SignVec::from(&[1, -2, 3][..]);
        assert_eq!(sign_vec.as_ref(), &vec![1, -2, 3]);
    }

    // Tests for Default for SignVec<T>
    #[test]
    fn test_default() {
        let sign_vec: SignVec<i32> = SignVec::default();
        assert!(sign_vec.vals.is_empty());
        // Further checks can include testing the default state of pos and neg if applicable.
    }

    // Tests for Extend<&T> for SignVec<T>
    #[test]
    fn test_extend_ref() {
        let mut sign_vec = SignVec::default();
        sign_vec.extend(&[1, -2, 3]);
        assert_eq!(sign_vec.vals, vec![1, -2, 3]);
        // Additional checks can verify the correct state of pos and neg.
    }

    // Tests for Extend<T> for SignVec<T>
    #[test]
    fn test_extend_owned() {
        let mut sign_vec = SignVec::default();
        sign_vec.extend(vec![1, -2, 3]);
        assert_eq!(sign_vec.vals, vec![1, -2, 3]);
        // Additional checks can verify the correct state of pos and neg.
    }

    // Tests for From<&[T]> for SignVec<T>
    #[test]
    fn test_from_slice() {
        let sign_vec = SignVec::from(&[1, -2, 3][..]);
        assert_eq!(sign_vec.vals, vec![1, -2, 3]);
    }

    // Tests for From<&[T; N]> for SignVec<T>
    #[test]
    fn test_from_array_ref() {
        let sign_vec = SignVec::from(&[1, -2, 3]);
        assert_eq!(sign_vec.vals, vec![1, -2, 3]);
    }

    // Tests for From<&mut [T]> for SignVec<T>
    #[test]
    fn test_from_mut_slice() {
        let sign_vec = SignVec::from(&mut [1, -2, 3][..]);
        assert_eq!(sign_vec.vals, vec![1, -2, 3]);
    }

    // Tests for From<&mut [T; N]> for SignVec<T>
    #[test]
    fn test_from_mut_array_ref() {
        let mut array = [1, -2, 3];
        let sign_vec = SignVec::from(&mut array);
        assert_eq!(sign_vec.vals, vec![1, -2, 3]);
    }

    // Tests for From<&Vec<T>> for SignVec<T>
    #[test]
    fn test_from_vec_ref() {
        let vec = vec![1, -2, 3];
        let sign_vec = SignVec::from(&vec);
        assert_eq!(sign_vec.vals, vec![1, -2, 3]);
    }

    // Tests for From<Vec<T>> for SignVec<T>
    #[test]
    fn test_from_vec() {
        let vec = vec![1, -2, 3];
        let sign_vec = SignVec::from(vec);
        assert_eq!(sign_vec.vals, vec![1, -2, 3]);
    }

    // Tests for From<[T; N]> for SignVec<T>
    #[test]
    fn test_from_array() {
        let array = [1, -2, 3];
        let sign_vec = SignVec::from(array);
        assert_eq!(sign_vec.vals, vec![1, -2, 3]);
    }
    // Test FromIterator<T> for SignVec<T>
    #[test]
    fn from_iterator_owned() {
        let items = vec![1, -1, 2, -2];
        let sign_vec: SignVec<i32> = items.into_iter().collect();
        assert_eq!(sign_vec.vals, vec![1, -1, 2, -2]);
        // Additional checks can be added for pos and neg sets.
    }

    // Test FromIterator<&T> for SignVec<T>
    #[test]
    fn from_iterator_ref() {
        let items = [1, -1, 2, -2];
        let sign_vec: SignVec<i32> = items.iter().collect();
        assert_eq!(sign_vec.vals, vec![1, -1, 2, -2]);
        // Additional checks can be added for pos and neg sets.
    }

    // Test IntoIterator for SignVec<T> (owned iteration)
    #[test]
    fn into_iterator_owned() {
        let sign_vec = SignVec::from(&[1, -1, 2, -2][..]);
        let collected: Vec<i32> = sign_vec.into_iter().collect();
        assert_eq!(collected, vec![1, -1, 2, -2]);
    }

    // Test IntoIterator for &SignVec<T> (immutable reference iteration)
    #[test]
    fn into_iterator_ref() {
        let sign_vec = SignVec::from(&[1, -1, 2, -2][..]);
        let collected: Vec<&i32> = (&sign_vec).into_iter().collect();
        assert_eq!(collected, vec![&1, &-1, &2, &-2]);
    }

    // Test IntoIterator for &mut SignVec<T> (mutable reference iteration)
    #[test]
    fn into_iterator_mut_ref() {
        let mut sign_vec = SignVec::from(&[1, -1, 2, -2][..]);
        let mut collected: Vec<&mut i32> = (&mut sign_vec).into_iter().collect();
        // Perform a mutation as a demonstration.
        collected.iter_mut().for_each(|x| **x *= 2);
        assert_eq!(sign_vec.vals, vec![2, -2, 4, -4]);
    }
    // Test for Index trait implementation
    #[test]
    fn index_test() {
        let sv = SignVec::from(vec![1, -2, 3]);
        assert_eq!(sv[0], 1);
        assert_eq!(sv[1], -2);
        assert_eq!(sv[2], 3);
    }

    // Correction for the Deref test to properly use slicing
    #[test]
    fn deref_test() {
        let sv = SignVec::from(vec![1, -2, 3]);
        let slice: &[i32] = &sv; // Directly use Deref to get a slice
        assert_eq!(slice, &[1, -2, 3]);
    }

    // Test for Borrow trait implementation
    #[test]
    fn borrow_test() {
        let sv = SignVec::from(vec![1, -2, 3]);
        let slice: &[i32] = sv.borrow();
        assert_eq!(slice, &[1, -2, 3]);
    }

    // Test for Hash trait implementation
    #[test]
    fn hash_test() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let sv = SignVec::from(vec![1, -2, 3]);
        let mut hasher = DefaultHasher::new();
        sv.hash(&mut hasher);
        let hash = hasher.finish();

        let mut hasher_vec = DefaultHasher::new();
        vec![1, -2, 3].hash(&mut hasher_vec);
        let hash_vec = hasher_vec.finish();

        assert_eq!(hash, hash_vec);
    }

    // Test for Ord and PartialOrd trait implementations
    #[test]
    fn ord_partial_ord_test() {
        let sv1 = SignVec::from(vec![1, 2, 3]);
        let sv2 = SignVec::from(vec![1, 2, 4]);
        assert!(sv1 < sv2);
        assert!(sv2 > sv1);
    }

    // Test for Eq and PartialEq trait implementations
    #[test]
    fn eq_partial_eq_test() {
        let sv1 = SignVec::from(vec![1, 2, 3]);
        let sv2 = SignVec::from(vec![1, 2, 3]);
        let sv3 = SignVec::from(vec![1, 2, 4]);
        assert_eq!(sv1, sv2);
        assert_ne!(sv1, sv3);
    }

    #[test]
    fn partial_eq_with_others_test() {
        let sv = SignVec::from(vec![1, 2, 3]);

        // When comparing SignVec with a slice, ensure you're passing a reference to the slice,
        // not a double reference. The trait implementation expects a single reference.
        let slice: &[i32] = &[1, 2, 3];
        assert_eq!(sv, slice); // Use sv directly without an additional &, since PartialEq<&[U]> is implemented

        // Similarly, for a mutable slice, pass it as a single reference.
        let mut_slice: &mut [i32] = &mut [1, 2, 3];
        assert_eq!(sv, mut_slice); // Same as above, no double referencing

        // For comparing with a Vec<U>, the implementation should already handle it correctly.
        assert_eq!(sv, vec![1, 2, 3]); // Direct comparison with Vec<U> is supported
    }
}