use crate::{
    iter::{Iter, IterMut},
    AsDyn,
};
use std::{
    alloc::{alloc, dealloc, Layout},
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::{self, drop_in_place, NonNull, Pointee},
};

/// Contigous type-erased append-only vector
///
/// `Dyn` shall be `dyn Trait`
pub struct FuseBox<Dyn>
where
    Dyn: ?Sized,
{
    headers: Vec<Header<Dyn>>,
    inner: NonNull<u8>,
    max_align: usize,
    len_bytes: usize,
    cap_bytes: usize,
    _tag: PhantomData<Dyn>,
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

unsafe impl<Dyn> Send for FuseBox<Dyn> where Dyn: ?Sized {}

impl<Dyn> FuseBox<Dyn>
where
    Dyn: ?Sized,
{
    /// Creates a new [`FuseBox<Dyn>`].
    pub fn new() -> Self {
        Self {
            headers: Default::default(),
            inner: std::ptr::NonNull::dangling(),
            max_align: 0,
            len_bytes: 0,
            cap_bytes: 0,
            _tag: Default::default(),
        }
    }

    /// Returns the length of this [`FuseBox<Dyn>`] in items.
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    fn realloc(&mut self, min_size: usize) {
        if self.cap_bytes == 0 {
            unsafe {
                let layout = Layout::from_size_align_unchecked(min_size, self.max_align);
                self.cap_bytes = layout.pad_to_align().size();
                let new = alloc(layout);
                self.inner = NonNull::new_unchecked(new);
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
            std::ptr::copy(old.as_ptr(), new, self.len_bytes);
            self.inner = NonNull::new_unchecked(new);
            dealloc(old.as_ptr(), old_layout);
        }
    }

    /// Appends an element to the vector.
    ///
    /// # Preconditions
    ///
    /// `meta` MUST be derived from the same value that's being appended.
    ///
    /// # Note
    ///
    /// this method does not require that `T` impls [`Send`], making it unsound to send this
    /// instance of [`FuseBox`] across thread after pushing a `T: !Send`
    pub unsafe fn push_unsafe<T>(&mut self, v: T, meta: <Dyn as Pointee>::Metadata)
    where
        T: 'static,
    {
        let layout = Layout::new::<T>();
        let header = self.make_header(layout, meta);
        let offset = header.offset;
        let size = header.size;
        if layout.size() == 0 {
            unsafe { self.inner.as_ptr().add(offset).cast::<T>().write(v) }
            self.headers.push(header);
        } else {
            if self.max_align < layout.align() {
                self.max_align = layout.align()
            }
            if self.cap_bytes - self.len_bytes < layout.size() {
                self.realloc(layout.size())
            }

            unsafe { self.inner.as_ptr().add(offset).cast::<T>().write(v) }
            self.headers.push(header);
        }
        self.len_bytes = offset + size;
    }

    fn make_header(&mut self, layout: Layout, meta: <Dyn as Pointee>::Metadata) -> Header<Dyn> {
        if self.len() != 0 {
            let Header {
                offset,
                size,
                meta: _,
            } = self.headers[self.len() - 1];
            let offset = round_up(offset + size, layout.align());
            Header {
                offset,
                size: layout.size(),
                meta,
            }
        } else {
            Header {
                offset: 0,
                size: layout.size(),
                meta,
            }
        }
    }

    /// Safely appends an element to the vector.
    ///
    /// Guarantees that metadata matches the type by requiring that [`AsDyn`] is implemented for T
    pub fn push<T>(&mut self, v: T)
    where
        T: 'static,
        T: Send,
        T: AsDyn<Dyn>,
        Dyn: 'static,
    {
        unsafe {
            let meta = ptr::metadata(v.as_dyn());
            self.push_unsafe(v, meta)
        }
    }

    /// Requires that `T:` [`Send`]
    pub unsafe fn push_safer<T>(&mut self, v: T, meta: <Dyn as Pointee>::Metadata)
    where
        T: 'static,
        T: Send,
    {
        unsafe { self.push_unsafe(v, meta) }
    }

    pub(crate) fn get_raw(&self, n: usize) -> *mut Dyn {
        assert!(n <= self.len());
        let Header {
            offset,
            size: _,
            meta,
        } = self.headers[n];
        unsafe {
            let ptr = self.inner.as_ptr().add(offset).cast();
            ptr::from_raw_parts_mut::<Dyn>(ptr, meta)
        }
    }

    /// Retrieves `&mut Dyn` from [`FuseBox`].
    pub fn get_mut(&mut self, n: usize) -> Option<&mut Dyn> {
        if self.len() <= n {
            return None;
        }
        unsafe { Some(&mut *self.get_raw(n)) }
    }

    /// Retrieves `&Dyn` from [`FuseBox`].
    pub fn get(&self, n: usize) -> Option<&Dyn> {
        if self.len() <= n {
            return None;
        }
        unsafe { Some(&*self.get_raw(n)) }
    }

    /// Returns an iterator over `&Dyn` stored in this [`FuseBox`]
    pub fn iter<'f>(&'f self) -> Iter<'f, Dyn> {
        Iter::new(self)
    }

    /// Returns an iterator over `&mut Dyn` stored in this [`FuseBox`].
    pub fn iter_mut<'f>(&'f mut self) -> IterMut<'f, Dyn> {
        IterMut::new(self)
    }
}

impl<Dyn> Index<usize> for FuseBox<Dyn> {
    type Output = Dyn;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.get_raw(index) }
    }
}

impl<Dyn> IndexMut<usize> for FuseBox<Dyn> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.get_raw(index) }
    }
}

struct Header<Dyn>
where
    Dyn: ?Sized,
{
    offset: usize,
    size: usize,
    meta: <Dyn as Pointee>::Metadata,
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
