//! Implementation of `Filter` for certain non-probabilistic data structures. This can be helpful
//! for debugging and performance comparisons.
use std::collections::HashSet;
use std::convert::Infallible;
use std::hash::{BuildHasher, Hash};

use crate::filters::Filter;

impl<T, S> Filter<T> for HashSet<T, S>
where
    T: Clone + Eq + Hash,
    S: BuildHasher,
{
    type InsertErr = Infallible;

    fn clear(&mut self) {
        self.clear();
    }

    fn insert(&mut self, obj: &T) -> Result<bool, Self::InsertErr> {
        Ok(self.insert(obj.clone()))
    }

    fn union(&mut self, other: &Self) -> Result<(), Self::InsertErr> {
        unimplemented!()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn query(&self, obj: &T) -> bool {
        self.contains(obj)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::convert::Infallible;

    use crate::filters::Filter;

    #[test]
    fn hashset() {
        let set: &mut dyn Filter<u64, InsertErr = Infallible> = &mut HashSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert!(!set.query(&42));

        assert!(set.insert(&42).unwrap());
        assert!(!set.insert(&42).unwrap());
        assert!(!set.is_empty());
        assert_eq!(set.len(), 1);
        assert!(set.query(&42));
        assert!(!set.query(&13));
    }
}
