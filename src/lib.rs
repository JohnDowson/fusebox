#![feature(ptr_metadata)]
#![feature(unsize)]
#![warn(clippy::dbg_macro)]

//! # Mostly safe and sound append-only collection of trait objects
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

pub mod fuse;
pub mod iter;

pub use fuse::FuseBox;

#[cfg(test)]
mod test;
