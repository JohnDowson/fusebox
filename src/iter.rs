use crate::{fuse::FuseBox, Size};

pub struct Iter<'f, Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Size,
    <Sz as TryFrom<usize>>::Error: std::fmt::Debug,
{
    fused: &'f FuseBox<Dyn, Sz>,
    n: usize,
}

impl<'f, Dyn, Sz> Iter<'f, Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Size,
    <Sz as TryFrom<usize>>::Error: std::fmt::Debug,
{
    pub(crate) fn new(fused: &'f FuseBox<Dyn, Sz>) -> Self {
        Self { fused, n: 0 }
    }
}

impl<'f, Dyn, Sz> Iterator for Iter<'f, Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Size,
    <Sz as TryFrom<usize>>::Error: std::fmt::Debug,
{
    type Item = &'f Dyn;
    fn next(&mut self) -> Option<Self::Item> {
        if self.n == self.fused.len() {
            return None;
        }
        let r = self.fused.get(self.n);
        self.n += 1;
        Some(r)
    }
}

pub struct IterMut<'f, Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Size,
    <Sz as TryFrom<usize>>::Error: std::fmt::Debug,
{
    fused: &'f mut FuseBox<Dyn, Sz>,
    n: usize,
}

impl<'f, Dyn, Sz> IterMut<'f, Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Size,
    <Sz as TryFrom<usize>>::Error: std::fmt::Debug,
{
    pub(crate) fn new(fused: &'f mut FuseBox<Dyn, Sz>) -> Self {
        Self { fused, n: 0 }
    }
}

impl<'f, Dyn, Sz> Iterator for IterMut<'f, Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Size,
    <Sz as TryFrom<usize>>::Error: std::fmt::Debug,
{
    type Item = &'f mut Dyn;
    fn next(&mut self) -> Option<Self::Item> {
        if self.n == self.fused.len() {
            return None;
        }
        let r = self.fused.get_raw(self.n);
        self.n += 1;
        Some(unsafe { &mut *r })
    }
}
