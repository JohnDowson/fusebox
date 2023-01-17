#![feature(ptr_metadata)]
#![feature(unsize)]
#![warn(clippy::dbg_macro)]

//! # fusebox
//!
//! Mostly safe and sound append-only collection of trait objects
//!
//! # Why?
//!
//! This avoids extra indirection of [`Vec<Box<dyn Trait>>`]
//!
//! # Usage
//!
//! ```
//! # use std::fmt::Debug;
//! # use fusebox::FuseBox;
//! # #[derive(Debug)]
//! # struct MyStruct {}
//! let value = MyStruct {};
//! let mut fb = FuseBox::<dyn Debug>::default();
//! fb.push(value);
//! ```

macro_rules! impl_iter {
    ($iter:tt $(, $mut:tt)?) => {
        pub struct $iter<'f, Dyn>
        where
            Dyn: ?Sized,
        {
            fused: &'f $($mut)? FuseBox<Dyn>,
            n: usize,
        }

        impl<'f, Dyn> $iter<'f, Dyn>
        where
            Dyn: ?Sized,
        {
            pub(crate) fn new(fused: &'f $($mut)? FuseBox<Dyn>) -> Self {
                Self { fused, n: 0 }
            }
        }

        impl<'f, Dyn> Iterator for $iter<'f, Dyn>
        where
            Dyn: ?Sized,
        {
            type Item = &'f $($mut)? Dyn;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.n == self.fused.len() {
                    return None;
                }
                let r = unsafe { & $($mut)? *self.fused.get_raw(self.n) };
                self.n += 1;
                Some(r)
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.fused.len();
                (len, Some(len))
            }

            #[inline]
            fn count(self) -> usize
            where
                Self: Sized,
            {
                self.fused.len()
            }

            #[inline]
            fn last(mut self) -> Option<Self::Item>
            where
                Self: Sized,
            {
                self.n = self.fused.len() - 1;
                self.next()
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                self.n = n;
                self.next()
            }
        }

        impl<'f, Dyn> ExactSizeIterator for $iter<'f, Dyn>
        where
            Dyn: ?Sized,
        {
            #[inline]
            fn len(&self) -> usize {
                self.fused.len()
            }
        }
    };
}

pub mod fuse;
pub mod inline_meta;

pub use fuse::FuseBox;
