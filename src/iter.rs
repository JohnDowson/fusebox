use crate::fuse::FuseBox;

pub struct Iter<'f, Dyn>
where
    Dyn: ?Sized,
{
    fused: &'f FuseBox<Dyn>,
    n: usize,
}

impl<'f, Dyn> Iter<'f, Dyn>
where
    Dyn: ?Sized,
{
    pub(crate) fn new(fused: &'f FuseBox<Dyn>) -> Self {
        Self { fused, n: 0 }
    }
}

impl<'f, Dyn> Iterator for Iter<'f, Dyn>
where
    Dyn: ?Sized,
{
    type Item = &'f Dyn;
    fn next(&mut self) -> Option<Self::Item> {
        if self.n == self.fused.len() {
            return None;
        }
        let r = self.fused.get(self.n);
        self.n += 1;
        r
    }
}

pub struct IterMut<'f, Dyn>
where
    Dyn: ?Sized,
{
    fused: &'f mut FuseBox<Dyn>,
    n: usize,
}

impl<'f, Dyn> IterMut<'f, Dyn>
where
    Dyn: ?Sized,
{
    pub(crate) fn new(fused: &'f mut FuseBox<Dyn>) -> Self {
        Self { fused, n: 0 }
    }
}

impl<'f, Dyn> Iterator for IterMut<'f, Dyn>
where
    Dyn: ?Sized,
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
