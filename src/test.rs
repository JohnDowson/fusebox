
use crate::FuseBox;
use std::{fmt::Debug, ops::ShlAssign};
#[test]
fn test() {
    let mut fb = FuseBox::<dyn Debug>::default();

    let v = 16u64;
    fb.push(v);

    let v = 1u8;
    fb.push(v);

    let v = 2u8;
    fb.push(v);

    let v = [1u8; 5];
    fb.push(v);

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
    fb.push(v);

    let v = 1u8;
    fb.push(v);

    let v = 2u8;
    fb.push(v);

    let v = 5u32;
    fb.push(v);

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
