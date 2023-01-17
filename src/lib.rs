#![feature(ptr_metadata)]
#![feature(unsize)]
#![feature(core_intrinsics)]
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

pub mod fuse;

pub use fuse::FuseBox;
