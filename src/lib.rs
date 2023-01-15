#![feature(ptr_metadata)]

//! # Mostly safe and sound append-only collection of trait objects
//!
//! # Why?
//!
//! This avoids extra indirection of [`Vec<dyn Trait>`]

pub mod fuse;
pub mod iter;

/// Safe wrapper for [`FuseBox::push()`]
///
/// # Usage
///
/// ```
/// #![feature(ptr_metadata)]
///
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
            $fuse.push($val, meta);
        }
    };
}

/// Convinience alias for [`fuse::FuseBox<Dyn, usize>`]
pub type FuseBox<Dyn> = fuse::FuseBox<Dyn, usize>;

#[cfg(test)]
mod test {
    use crate::FuseBox;
    use std::fmt::Debug;

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
}
