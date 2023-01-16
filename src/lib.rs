#![feature(ptr_metadata)]

//! # Mostly safe and sound append-only collection of trait objects
//!
//! # Why?
//!
//! This avoids extra indirection of [`Vec<dyn Trait>`]
//!
//! # Usage
//!
//! For foreigin types you have to use [`push!`] macro.
//! ```
//! #![feature(ptr_metadata)]
//! # use std::fmt::Debug;
//! # use fusebox::FuseBox;
//! # use fusebox::push;
//! let value = 69420usize;
//! let mut fb = FuseBox::<dyn Debug>::default();
//! push!(value, fb, Debug);
//! ```
//!
//! For your own types:
//! ```
//! # use std::fmt::Debug;
//! # use fusebox::FuseBox;
//! # use fusebox::push;
//! # use fusebox::impl_as_dyn;
//! # #[derive(Debug)]
//! # struct MyStruct {}
//! impl_as_dyn!(MyStruct => dyn Debug + 'static);
//! let value = MyStruct {};
//! let mut fb = FuseBox::default();
//! fb.push(value);
//! ```

pub mod fuse;
pub mod iter;

/// Safe wrapper for [`FuseBox::push_safer`]
///
/// # Usage
///
/// ```
/// #![feature(ptr_metadata)]
/// # use std::fmt::Debug;
/// # use fusebox::FuseBox;
/// # use fusebox::push;
/// let value = 69420usize;
/// let mut fb = FuseBox::<dyn Debug>::default();
/// push!(value, fb, Debug);
/// ```
#[macro_export]
macro_rules! push {
    ($val:expr, $fuse:expr, $tr:path) => {
        let meta = ::std::ptr::metadata((&$val) as &dyn $tr);
        unsafe {
            $fuse.push_safer($val, meta);
        }
    };
}

/// Helper macro for implementing [`AsDyn`] for use with safe [`FuseBox::push`]
/// # Usage
///
/// ```
/// # use std::fmt::Debug;
/// # use fusebox::impl_as_dyn;
/// # use fusebox::AsDyn;
/// ##[derive(Debug)]
/// # struct MyStruct {}
/// impl_as_dyn!(MyStruct => dyn Debug + 'static);
/// ```
#[macro_export]
macro_rules! impl_as_dyn {
    ($typ:ty => $tra:ty) => {
        impl fusebox::AsDyn<$tra> for $typ {
            fn as_dyn(&self) -> &$tra {
                self as &$tra
            }
        }
    };
}

pub use fuse::FuseBox;

pub trait AsDyn<Dyn>
where
    Dyn: ?Sized,
{
    fn as_dyn(&self) -> &Dyn;
}

#[cfg(test)]
mod test {
    use crate::FuseBox;
    use std::{fmt::Debug, ops::ShlAssign};

    #[test]
    fn test() {
        let mut fb = FuseBox::<dyn Debug>::default();

        let v = 16u64;
        push!(v, fb, Debug);

        let v = 1u8;
        push!(v, fb, Debug);

        let v = 2u8;
        push!(v, fb, Debug);

        let v = [1u8; 5];
        push!(v, fb, Debug);

        for v in fb.iter() {
            println!("{v:?}")
        }
    }

    #[test]
    fn mutate() {
        trait ShlDebug: ShlAssign<u8> + Debug {}
        impl<T> ShlDebug for T where T: ShlAssign<u8> + Debug {}
        let mut fb = FuseBox::<dyn ShlDebug>::default();

        let v = 16u64;
        push!(v, fb, ShlDebug);

        let v = 1u8;
        push!(v, fb, ShlDebug);

        let v = 2u8;
        push!(v, fb, ShlDebug);

        let v = 5u32;
        push!(v, fb, ShlDebug);

        for v in fb.iter() {
            println!("{v:?}")
        }
        println!();
        for v in fb.iter_mut() {
            v.shl_assign(1);
        }
        for v in fb.iter() {
            println!("{v:?}")
        }
    }
}
