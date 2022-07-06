// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Every codebase needs a `util` module.

use std::cmp::Ordering;
use std::ops::{Bound, IndexMut, Range, RangeBounds};
use std::ptr;

#[cfg(feature = "pool")]
pub(crate) use refpool::{PoolClone, PoolDefault};

// The `Ref` type is an alias for either `Rc` or `Arc`, user's choice.

// `Arc` without refpool
// #[cfg(all(threadsafe))]
// pub(crate) use crate::fakepool::{Arc as PoolRef, Pool, PoolClone, PoolDefault};

// `Ref` == `Arc` when threadsafe
#[cfg(threadsafe)]
pub(crate) type Ref<A> = std::sync::Arc<A>;

// `Rc` without refpool
// #[cfg(all(not(threadsafe), not(feature = "pool")))]
// pub(crate) use crate::fakepool::{Pool, PoolClone, PoolDefault, Rc as PoolRef};

// `Rc` with refpool
// #[cfg(all(not(threadsafe), feature = "pool"))]
// pub(crate) type PoolRef<A> = refpool::PoolRef<A>;
// #[cfg(all(not(threadsafe), feature = "pool"))]
// pub(crate) type Pool<A> = refpool::Pool<A>;

// `Ref` == `Rc` when not threadsafe
#[cfg(not(threadsafe))]
pub(crate) type Ref<A> = std::rc::Rc<A>;

pub(crate) fn clone_ref<A>(r: Ref<A>) -> A
where
    A: Clone,
{
    Ref::try_unwrap(r).unwrap_or_else(|r| (*r).clone())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Side {
    Left,
    Right,
}

/// Swap two values of anything implementing `IndexMut`.
///
/// Like `slice::swap`, but more generic.
#[allow(unsafe_code)]
pub(crate) fn swap_indices<V>(vector: &mut V, a: usize, b: usize)
where
    V: IndexMut<usize>,
    V::Output: Sized,
{
    if a == b {
        return;
    }
    // so sorry, but there's no implementation for this in std that's
    // sufficiently generic
    let pa: *mut V::Output = &mut vector[a];
    let pb: *mut V::Output = &mut vector[b];
    unsafe {
        ptr::swap(pa, pb);
    }
}

#[allow(dead_code)]
pub(crate) fn linear_search_by<'a, A, I, F>(iterable: I, mut cmp: F) -> Result<usize, usize>
where
    A: 'a,
    I: IntoIterator<Item = &'a A>,
    F: FnMut(&A) -> Ordering,
{
    let mut pos = 0;
    for value in iterable {
        match cmp(value) {
            Ordering::Equal => return Ok(pos),
            Ordering::Greater => return Err(pos),
            Ordering::Less => {}
        }
        pos += 1;
    }
    Err(pos)
}

pub(crate) fn to_range<R>(range: &R, right_unbounded: usize) -> Range<usize>
where
    R: RangeBounds<usize>,
{
    let start_index = match range.start_bound() {
        Bound::Included(i) => *i,
        Bound::Excluded(i) => *i + 1,
        Bound::Unbounded => 0,
    };
    let end_index = match range.end_bound() {
        Bound::Included(i) => *i + 1,
        Bound::Excluded(i) => *i,
        Bound::Unbounded => right_unbounded,
    };
    start_index..end_index
}

// macro_rules! def_pool {
//     ($name:ident<$($arg:tt),*>, $pooltype:ty) => {
//         /// A memory pool for the appropriate node type.
//         pub struct $name<$($arg,)*>(Pool<$pooltype>);

//         impl<$($arg,)*> $name<$($arg,)*> {
//             /// Create a new pool with the given size.
//             pub fn new(size: usize) -> Self {
//                 Self(Pool::new(size))
//             }

//             /// Fill the pool with preallocated chunks.
//             pub fn fill(&self) {
//                 self.0.fill();
//             }

//             ///Get the current size of the pool.
//             pub fn pool_size(&self) -> usize {
//                 self.0.get_pool_size()
//             }
//         }

//         impl<$($arg,)*> Default for $name<$($arg,)*> {
//             fn default() -> Self {
//                 Self::new($crate::config::POOL_SIZE)
//             }
//         }

//         impl<$($arg,)*> Clone for $name<$($arg,)*> {
//             fn clone(&self) -> Self {
//                 Self(self.0.clone())
//             }
//         }
//     };
// }

// TODO require Default and Clone impl
pub(crate) trait PoolLike {
    type Value;
    type PoolRef;

    /// Create a new pool with the given size.
    /// The size is advisable.
    fn new(size: usize) -> Self;

    /// Fill the pool with preallocated chunks?
    fn fill(&self);

    /// Return the current pool size?
    fn pool_size(&self) -> usize;

    fn new_ref(&self, value: Self::Value) -> Self::PoolRef;

    fn ptr_eq(left: &Self::PoolRef, right: &Self::PoolRef) -> bool;
}

pub struct Pool<T> {
    inner: refpool::Pool<T>,
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        PoolLike::new(crate::config::POOL_SIZE)
    }
}

impl<T> PoolLike for Pool<T> {
    type Value = T;
    type PoolRef = refpool::PoolRef<T>;

    fn new(size: usize) -> Self {
        Self {
            inner: refpool::Pool::new(size),
        }
    }

    fn fill(&self) {
        self.inner.fill();
    }

    fn pool_size(&self) -> usize {
        self.inner.get_pool_size()
    }

    fn new_ref(&self, value: Self::Value) -> Self::PoolRef {
        refpool::PoolRef::new(&self.inner, value)
    }

    fn ptr_eq(left: &Self::PoolRef, right: &Self::PoolRef) -> bool {
        refpool::PoolRef::ptr_eq(left, right)
    }
}

pub trait PoolLikeClone: PoolLike {
    fn make_mut<'a>(&self, this: &'a mut Self::PoolRef) -> &'a mut Self::Value;
    // fn unwrap_or_clone(&self, this: Self::PoolRef) -> Self::Value; //
}

pub trait PoolLikeDefault: PoolLike {
    fn default_ref(&self) -> Self::PoolRef;
}

impl<T: PoolDefault> PoolLikeDefault for Pool<T> {
    fn default_ref(&self) -> Self::PoolRef {
        refpool::PoolRef::default(&self.inner)
    }
}

impl<T: PoolClone> PoolLikeClone for Pool<T> {
    fn make_mut<'a>(&self, this: &'a mut Self::PoolRef) -> &'a mut T {
        refpool::PoolRef::make_mut(&self.inner, this)
    }

    // fn unwrap_or_clone(&self, this: Self::PoolRef) -> T {
    //     refpool::PoolRef::unwrap_or_clone(this)
    // }
}

pub(crate) use {refpool::PoolClone, refpool::PoolDefault, refpool::PoolRef};
