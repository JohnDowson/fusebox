use iter::{Iter, IterMut};
use std::{
    alloc::{alloc, dealloc, Layout},
    marker::Unsize,
    ops::{Index, IndexMut},
    ptr::{self, drop_in_place, NonNull, Pointee},
};

pub mod iter;

#[cfg(test)]
mod test;

#[derive(Clone, Copy)]
struct Header<Dyn>
where
    Dyn: ?Sized,
{
    offset: usize,
    meta: <Dyn as Pointee>::Metadata,
}

/// Contigous type-erased append-only vector
///
/// `Dyn` shall be `dyn Trait`
pub struct FuseBox<Dyn>
where
    Dyn: ?Sized,
{
    headers: Vec<Header<Dyn>>,
    inner: NonNull<u8>,
    last_size: usize,
    max_align: usize,
    len_bytes: usize,
    cap_bytes: usize,
}

impl<Dyn> Default for FuseBox<Dyn>
where
    Dyn: ?Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Dyn> Drop for FuseBox<Dyn>
where
    Dyn: ?Sized,
{
    fn drop(&mut self) {
        if self.cap_bytes != 0 {
            // Safety:
            // inner guaranteed to be valid here
            // values are guaranteed to be aligned
            unsafe {
                for val in self.iter_mut() {
                    drop_in_place(val);
                }
                dealloc(
                    self.inner.as_ptr(),
                    Layout::from_size_align_unchecked(self.cap_bytes, self.max_align),
                );
            }
        }
    }
}

unsafe impl<Dyn> Send for FuseBox<Dyn>
where
    Dyn: ?Sized,
    Dyn: Send,
{
}

unsafe impl<Dyn> Sync for FuseBox<Dyn>
where
    Dyn: ?Sized,
    Dyn: Sync,
{
}

impl<Dyn> FuseBox<Dyn>
where
    Dyn: ?Sized,
{
    #[must_use]
    /// Creates a new [`FuseBox<Dyn>`].
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            inner: std::ptr::NonNull::dangling(),
            last_size: 0,
            max_align: 0,
            len_bytes: 0,
            cap_bytes: 0,
        }
    }

    #[must_use]
    #[inline]
    /// Returns the length of this [`FuseBox<Dyn>`] in items.
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn realloc(&mut self, min_size: usize) {
        if self.cap_bytes == 0 {
            unsafe {
                let layout = Layout::from_size_align_unchecked(min_size, self.max_align);
                self.cap_bytes = layout.pad_to_align().size();
                let new = alloc(layout);
                self.inner = NonNull::new_unchecked(new);
            }
            return;
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
            let layout = Layout::from_size_align_unchecked(size, self.max_align).pad_to_align();
            self.cap_bytes = layout.size();
            let new = alloc(layout);
            std::ptr::copy(old.as_ptr(), new, self.len_bytes);
            self.inner = NonNull::new_unchecked(new);
            dealloc(old.as_ptr(), old_layout);
        }
    }

    #[inline]
    unsafe fn push_unsafe<T>(&mut self, v: T)
    where
        T: 'static,
        T: Unsize<Dyn>,
    {
        let as_dyn: &Dyn = &v;
        let meta = ptr::metadata(as_dyn);
        let layout = Layout::new::<T>();
        let header = self.make_header(layout, meta);
        let offset = header.offset;

        if layout.size() == 0 && layout.align() <= 1 {
            // Safety: offset guaranteed to be in-bounds
            unsafe { self.inner.as_ptr().add(offset).cast::<T>().write(v) }
            self.headers.push(header);
        } else {
            if self.max_align < layout.align() {
                self.max_align = layout.align();
            }
            if self.cap_bytes - offset < layout.size() {
                self.realloc(layout.size());
            }

            unsafe { self.inner.as_ptr().add(offset).cast::<T>().write(v) }
            self.headers.push(header);
        }
        self.last_size = layout.size();
        self.len_bytes = offset + layout.size();
    }

    #[inline]
    fn make_header(&mut self, layout: Layout, meta: <Dyn as Pointee>::Metadata) -> Header<Dyn> {
        if self.is_empty() {
            Header { offset: 0, meta }
        } else {
            let Header { offset, meta: _ } = self.headers[self.len() - 1];
            let offset = round_up(offset + self.last_size, layout.align());
            Header { offset, meta }
        }
    }

    #[inline]
    /// Appends an element to the vector.
    pub fn push<T>(&mut self, v: T)
    where
        T: 'static,
        T: Unsize<Dyn>,
        Dyn: 'static,
    {
        unsafe { self.push_unsafe(v) }
    }

    #[inline]
    pub(crate) unsafe fn get_raw(&self, n: usize) -> *mut Dyn {
        let Header { offset, meta } = self.headers[n];
        unsafe {
            let ptr = self.inner.as_ptr().add(offset).cast();
            ptr::from_raw_parts_mut::<Dyn>(ptr, meta)
        }
    }

    #[inline]
    /// Retrieves `&mut Dyn` from [`FuseBox`].
    pub fn get_mut(&mut self, n: usize) -> Option<&mut Dyn> {
        if self.len() <= n {
            return None;
        }
        unsafe { Some(&mut *self.get_raw(n)) }
    }

    #[inline]
    #[must_use]
    /// Retrieves `&Dyn` from [`FuseBox`].
    pub fn get(&self, n: usize) -> Option<&Dyn> {
        if self.len() <= n {
            return None;
        }
        unsafe { Some(&*self.get_raw(n)) }
    }

    #[must_use]
    /// Returns an iterator over `&Dyn` stored in this [`FuseBox`]
    pub fn iter(&'_ self) -> Iter<'_, Dyn> {
        Iter::new(self)
    }

    #[must_use]
    /// Returns an iterator over `&mut Dyn` stored in this [`FuseBox`].
    pub fn iter_mut(&'_ mut self) -> IterMut<'_, Dyn> {
        IterMut::new(self)
    }
}

impl<Dyn> Index<usize> for FuseBox<Dyn>
where
    Dyn: ?Sized,
{
    type Output = Dyn;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len());
        unsafe { &*self.get_raw(index) }
    }
}

impl<Dyn> IndexMut<usize> for FuseBox<Dyn>
where
    Dyn: ?Sized,
{
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len());
        unsafe { &mut *self.get_raw(index) }
    }
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
