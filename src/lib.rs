#![feature(ptr_metadata)]
mod fuse;
mod iter;

pub type FuseBox<Dyn, Sz = usize> = fuse::FuseBox<Dyn, Sz>;

#[cfg(test)]
mod test {
    use crate::FuseBox;
    use std::{fmt::Debug, ptr::metadata};

    #[test]
    fn push() {
        let mut fb = FuseBox::<dyn Debug>::default();

        let v = 16u64;
        fb.push(v, metadata((&v) as &dyn Debug));

        let v = 1u8;
        fb.push(v, metadata((&v) as &dyn Debug));

        let v = 1u8;
        fb.push(v, metadata((&v) as &dyn Debug));

        let v = 1u16;
        fb.push(v, metadata((&v) as &dyn Debug));

        for v in fb.iter() {
            println!("{v:?}")
        }
    }
}
