//! `OrderMap 1.x` doesn't implement `schemars::JsonSchema`, pending:
//!
//! <https://github.com/GREsau/schemars/pull/516>
//!
//! For now, we use `IndexMap` when the `schemars` feature is enabled.
//!
//! In general, this library should be built with the `"schemars"` feature
//! disabled.
//!
//! Tests rely on OrderMap as some assertions expect order to be preserved.

use core::hash::{BuildHasher, Hash};

use ordermap::Equivalent;

#[cfg(all(feature = "schemars", not(feature = "test")))]
pub use indexmap::map::Keys;
#[cfg(all(feature = "schemars", not(feature = "test")))]
pub use indexmap::IndexMap as Map;

#[cfg(any(not(feature = "schemars"), feature = "test"))]
pub use ordermap::map::Keys;
#[cfg(any(not(feature = "schemars"), feature = "test"))]
pub use ordermap::OrderMap as Map;

/// Order-preserving removal operations shared by both [`Map`] backends.
///
/// `ordermap::OrderMap` and `indexmap::IndexMap` diverge on the names of their
/// order-preserving removal methods -- `OrderMap` uses `remove`, while
/// `IndexMap` deprecated that in favour of `shift_remove`. This trait exposes a
/// single API that removes an entry while shifting the later entries down to
/// preserve order, regardless of which backend the `schemars` feature selects.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::{Map, MapOrderedRemove};
///
/// let mut map = Map::<u32, &str>::new();
/// map.insert(1, "a");
/// map.insert(2, "b");
///
/// assert_eq!(map.remove_ordered(&1), Some("a"));
/// assert_eq!(map.keys().copied().collect::<Vec<_>>(), vec![2]);
/// ```
pub trait MapOrderedRemove<K, V> {
    /// Removes `key` from the map, preserving the order of the remaining
    /// entries.
    ///
    /// Returns the removed value, or `None` if the key was not present.
    fn remove_ordered<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K> + ?Sized;
}

impl<K, V, S> MapOrderedRemove<K, V> for Map<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    fn remove_ordered<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        #[cfg(all(feature = "schemars", not(feature = "test")))]
        {
            self.shift_remove(key)
        }
        #[cfg(any(not(feature = "schemars"), feature = "test"))]
        {
            self.remove(key)
        }
    }
}
