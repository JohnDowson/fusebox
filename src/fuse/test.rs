use super::FuseBox;
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
fn silly() {
    let mut fb = FuseBox::<[u8]>::default();

    let v = [0; 2];
    fb.push(v);
    let v = [0; 4];
    fb.push(v);
    let v = [0; 8];
    fb.push(v);
    let v = [0; 16];
    fb.push(v);

    for v in fb.iter() {
        println!("{v:?}")
    }
}

#[test]
// https://github.com/JohnDowson/fusebox/issues/4
fn issue4() {
    let mut fb = FuseBox::<dyn Debug>::default();

    fb.push(42u8);
    fb.push(1337_u128);

    for v in fb.iter() {
        println!("{v:?}")
    }
}

#[allow(clippy::all)]
#[test]
// https://github.com/JohnDowson/fusebox/issues/5
fn issue5() {
    let mut x: FuseBox<dyn Debug> = FuseBox::new();
    x.push(0_u8);
    x.push(0_u8);

    x.push(0_u16);
    x.push(0_u8);
    x.push(0_u8);
    x.push(0_u8);
    x.push(0_u8);

    x.push(0_u16);
    x.push(0_u32);
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
