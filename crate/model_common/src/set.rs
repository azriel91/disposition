//! `OrderSet 1.x` doesn't implement `schemars::JsonSchema`, pending:
//!
//! <https://github.com/GREsau/schemars/pull/516>
//!
//! For now, we use an `IndexSet` when the `schemars` feature is enabled.
//!
//! In general, this library should be built with the `"schemars"` feature
//! disabled.
//!
//! Tests rely on `ordermap::Set` as some assertions expect order to be
//! preserved.

use core::hash::{BuildHasher, Hash};

use ordermap::Equivalent;

#[cfg(all(feature = "schemars", not(feature = "test")))]
pub use indexmap::IndexSet as Set;

#[cfg(any(not(feature = "schemars"), feature = "test"))]
pub use ordermap::OrderSet as Set;

/// Order-preserving removal operations shared by both [`Set`] backends.
///
/// `ordermap::OrderSet` and `indexmap::IndexSet` diverge on the names of their
/// order-preserving removal methods -- `OrderSet` uses `remove` /
/// `remove_index`, while `IndexSet` deprecated those in favour of
/// `shift_remove` / `shift_remove_index`. This trait exposes a single API that
/// removes an entry while shifting the later entries down to preserve order,
/// regardless of which backend the `schemars` feature selects.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::{Set, SetOrderedRemove};
///
/// let mut set = Set::<u32>::new();
/// set.insert(1);
/// set.insert(2);
/// set.insert(3);
///
/// assert!(set.remove_ordered(&2));
/// assert_eq!(set.remove_index_ordered(0), Some(1));
/// assert_eq!(set.iter().copied().collect::<Vec<_>>(), vec![3]);
/// ```
pub trait SetOrderedRemove<T> {
    /// Removes `value` from the set, preserving the order of the remaining
    /// entries.
    ///
    /// Returns `true` if the value was present.
    fn remove_ordered<Q>(&mut self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized;

    /// Removes the entry at `index`, preserving the order of the remaining
    /// entries.
    ///
    /// Returns the removed value, or `None` if `index` is out of bounds.
    fn remove_index_ordered(&mut self, index: usize) -> Option<T>;
}

impl<T, S> SetOrderedRemove<T> for Set<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    fn remove_ordered<Q>(&mut self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        #[cfg(all(feature = "schemars", not(feature = "test")))]
        {
            self.shift_remove(value)
        }
        #[cfg(any(not(feature = "schemars"), feature = "test"))]
        {
            self.remove(value)
        }
    }

    fn remove_index_ordered(&mut self, index: usize) -> Option<T> {
        #[cfg(all(feature = "schemars", not(feature = "test")))]
        {
            self.shift_remove_index(index)
        }
        #[cfg(any(not(feature = "schemars"), feature = "test"))]
        {
            self.remove_index(index)
        }
    }
}
