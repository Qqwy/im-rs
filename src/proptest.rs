//! Proptest strategies.
//!
//! These are only available when using the `proptest` feature flag.

use crate::OrdMap;
use proptest::collection::vec;
use proptest::strategy::{BoxedStrategy, Strategy, ValueTree};
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Range;

/// A strategy for an [`OrdMap`][OrdMap] of a given size.
///
/// # Examples
///
/// ```rust,no_run
/// # use ::proptest::proptest;
/// proptest! {
///     #[test]
///     fn proptest_works(ref m in ord_map(0..9999, ".*", 10..100)) {
///         assert!(m.len() < 100);
///         assert!(m.len() >= 10);
///     }
/// }
/// ```
///
/// [OrdMap]: ../struct.OrdMap.html
pub fn ord_map<K: Strategy + 'static, V: Strategy + 'static>(
    key: K,
    value: V,
    size: Range<usize>,
) -> BoxedStrategy<OrdMap<<K::Tree as ValueTree>::Value, <V::Tree as ValueTree>::Value>>
where
    <K::Tree as ValueTree>::Value: Ord + Clone,
    <V::Tree as ValueTree>::Value: Clone,
{
    ::proptest::collection::vec((key, value), size.clone())
        .prop_map(OrdMap::from)
        .prop_filter("OrdMap minimum size".to_owned(), move |m| {
            m.len() >= size.start
        })
        .boxed()
}
