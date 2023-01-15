use std::{
    alloc::{alloc, dealloc, Layout},
    fmt::Debug,
    marker::PhantomData,
    ptr::{self, addr_of_mut, Pointee},
};

use crate::iter::{Iter, IterMut};

pub struct FuseBox<Dyn, Sz>
where
    Dyn: ?Sized,
{
    inner: *mut u8,
    max_align: usize,
    len_items: usize,
    len_bytes: usize,
    cap_bytes: usize,
    _tag: PhantomData<(Box<Dyn>, Sz)>,
}

impl<Dyn, Sz> Default for FuseBox<Dyn, Sz>
where
    Dyn: ?Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Dyn, Sz> Drop for FuseBox<Dyn, Sz>
where
    Dyn: ?Sized,
{
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                dealloc(
                    self.inner,
                    Layout::from_size_align_unchecked(self.cap_bytes, self.max_align),
                );
            }
        }
    }
}

unsafe impl<Dyn, Sz> Send for FuseBox<Dyn, Sz> where Dyn: ?Sized {}

impl<Dyn, Sz> FuseBox<Dyn, Sz>
where
    Dyn: ?Sized,
{
    pub fn new() -> Self {
        Self {
            inner: std::ptr::null_mut(),
            max_align: 0,
            len_items: 0,
            len_bytes: 0,
            cap_bytes: 0,
            _tag: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.len_items
    }

    fn realloc(&mut self, min_size: usize) {
        if self.inner.is_null() {
            unsafe {
                let layout = Layout::from_size_align_unchecked(min_size, self.max_align);
                self.cap_bytes = layout.pad_to_align().size();
                let new = alloc(layout);
                self.inner = new;
            }
        }
        let old = self.inner;
        let old_layout =
            unsafe { Layout::from_size_align_unchecked(self.cap_bytes, self.max_align) };
        let size = if self.cap_bytes == 0 {
            min_size
        } else {
            self.cap_bytes
                .checked_mul(2)
                .and_then(|s| s.checked_add(min_size))
                .expect("New capacity overflowed usize")
        };
        unsafe {
            let layout = Layout::from_size_align_unchecked(size, self.max_align);
            self.cap_bytes = layout.pad_to_align().size();
            let new = alloc(layout);
            std::ptr::copy(old, new, self.len_bytes);
            self.inner = new;
            dealloc(old, old_layout);
        }
    }

    pub fn push<T: 'static>(&mut self, v: T, meta: <Dyn as Pointee>::Metadata)
    where
        Sz: Copy,
        Sz: TryFrom<usize>,
        <Sz as TryFrom<usize>>::Error: Debug,
        Sz: Into<usize>,
    {
        let layout = Layout::new::<Unbox<T, Dyn, Sz>>();
        if self.max_align < layout.align() {
            self.max_align = layout.align()
        }
        if self.cap_bytes - self.len_bytes < layout.size() {
            self.realloc(layout.size())
        }
        let v = Unbox {
            size: layout.size().try_into().expect("Element too large"),
            meta,
            inner: v,
        };
        if let Some(n) = self.len_items.checked_sub(1) {
            let last_size = self.get_size(n);
            *last_size = round_up((*last_size).into(), layout.align())
                .try_into()
                .expect("Element too large");
        }

        self.len_bytes = round_up(self.len_bytes, layout.align());
        unsafe {
            self.inner
                .add(self.len_bytes)
                .cast::<Unbox<T, Dyn, Sz>>()
                .write(v)
        }
        self.len_bytes += layout.size();
        self.len_items += 1;
    }

    fn get_size(&mut self, n: usize) -> &mut Sz
    where
        Sz: Into<usize>,
        Sz: Copy,
    {
        assert!(
            n <= self.len_items,
            "Assertion failed ({n}<{})",
            self.len_items
        );

        let mut item = self.inner;
        for _ in 0..n {
            unsafe {
                let size = (&*item.cast::<Unbox<(), Dyn, Sz>>()).size;
                item = item.add(size.into())
            }
        }
        unsafe { &mut (*item.cast::<Unbox<(), Dyn, Sz>>()).size }
    }

    pub(crate) fn get_raw(&self, n: usize) -> *const Dyn
    where
        Sz: Into<usize>,
        Sz: Copy,
    {
        assert!(n <= self.len_items);
        let mut item = self.inner;
        for _ in 0..n {
            unsafe {
                let size = (&*item.cast::<Unbox<(), Dyn, Sz>>()).size;
                item = item.add(size.into())
            }
        }
        unsafe {
            let item = &mut *item.cast::<Unbox<(), Dyn, Sz>>();
            let meta = item.meta;
            let ptr = &item.inner;
            ptr::from_raw_parts::<Dyn>(ptr, meta)
        }
    }

    pub(crate) fn get_raw_mut(&mut self, n: usize) -> *mut Dyn
    where
        Sz: Into<usize>,
        Sz: Copy,
    {
        assert!(n <= self.len_items);
        let mut item = self.inner;
        for _ in 0..n {
            unsafe {
                let size = (&*item.cast::<Unbox<(), Dyn, Sz>>()).size;
                item = item.add(size.into())
            }
        }
        unsafe {
            let item = &mut *item.cast::<Unbox<(), Dyn, Sz>>();
            let meta = item.meta;
            let ptr = &mut item.inner;
            ptr::from_raw_parts_mut::<Dyn>(ptr, meta)
        }
    }

    pub fn get_mut(&mut self, n: usize) -> &mut Dyn
    where
        Sz: Into<usize>,
        Sz: Copy,
    {
        unsafe { &mut *self.get_raw_mut(n) }
    }

    pub fn get(&self, n: usize) -> &Dyn
    where
        Sz: Into<usize>,
        Sz: Copy,
    {
        unsafe { &*self.get_raw(n) }
    }

    pub fn iter<'f>(&'f self) -> Iter<'f, Dyn, Sz> {
        Iter::new(self)
    }

    pub fn iter_mut<'f>(&'f mut self) -> IterMut<'f, Dyn, Sz> {
        IterMut::new(self)
    }
}

#[repr(C)]
struct Unbox<T, Dyn, Sz>
where
    Dyn: ?Sized,
{
    size: Sz,
    meta: <Dyn as Pointee>::Metadata,
    inner: T,
}

fn round_up(n: usize, m: usize) -> usize {
    if m == 0 {
        n
    } else {
        let rem = n % m;
        if rem == 0 {
            n
        } else {
            n + m - rem
        }
    }
}
