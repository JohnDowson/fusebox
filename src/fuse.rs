use crate::{
    iter::{Iter, IterMut},
    AsDyn,
};
use std::{
    alloc::{alloc, dealloc, Layout},
    fmt::Debug,
    marker::PhantomData,
    ptr::{self, addr_of_mut, drop_in_place, Pointee},
};

/// Contigous type-erased append-only vector
///
///
/// `Dyn` shall be `dyn Trait`
///
///
/// `Sz` shall be an unsigned integer no larger than [`usize`]
pub struct FuseBox<Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Into<usize>,
    Sz: Copy,
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
    Sz: Into<usize>,
    Sz: Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Dyn, Sz> Drop for FuseBox<Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Into<usize>,
    Sz: Copy,
{
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                for val in self.iter_mut() {
                    drop_in_place(val);
                }
                dealloc(
                    self.inner,
                    Layout::from_size_align_unchecked(self.cap_bytes, self.max_align),
                );
            }
        }
    }
}

unsafe impl<Dyn, Sz> Send for FuseBox<Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Into<usize>,
    Sz: Copy,
{
}

impl<Dyn, Sz> FuseBox<Dyn, Sz>
where
    Dyn: ?Sized,
    Sz: Into<usize>,
    Sz: Copy,
{
    /// Creates a new [`FuseBox<Dyn, Sz>`].
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

    /// Returns the length of this [`FuseBox<Dyn, Sz>`] in items.
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

    /// Safely appends an element to the vector.
    ///
    /// Guarantees that metadata matches the type by requiring that [`AsDyn`] is implemented for T
    ///
    /// # Panics
    ///
    /// Panics if size of new element is larger than `Sz::MAX`.
    pub fn push<T>(&mut self, v: T)
    where
        T: 'static,
        T: Send,
        T: AsDyn<Dyn>,
        Sz: TryFrom<usize>,
        <Sz as TryFrom<usize>>::Error: Debug,
    {
        unsafe {
            let meta = ptr::metadata(v.as_dyn());
            self.push_unsafe(v, meta)
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
    ///
    /// # Panics
    ///
    /// Panics if size of new element is larger than `Sz::MAX`.
    pub unsafe fn push_unsafe<T>(&mut self, v: T, meta: <Dyn as Pointee>::Metadata)
    where
        T: 'static,
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

    fn get_size(&mut self, n: usize) -> &mut Sz {
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

    pub(crate) fn get_raw(&self, n: usize) -> *mut Dyn {
        assert!(n <= self.len_items);
        let mut item = self.inner;
        for _ in 0..n {
            unsafe {
                let size = (&*item.cast::<Unbox<(), Dyn, Sz>>()).size;
                item = item.add(size.into())
            }
        }
        unsafe {
            let item = item.cast::<Unbox<(), Dyn, Sz>>();
            let meta = (&*item).meta;
            let ptr = addr_of_mut!((*item).inner);
            ptr::from_raw_parts_mut::<Dyn>(ptr, meta)
        }
    }

    /// Retrieves `&mut Dyn` from [`FuseBox`].
    ///
    /// # Panics
    ///
    /// Panics when `n >= len`
    pub fn get_mut(&mut self, n: usize) -> &mut Dyn {
        unsafe { &mut *self.get_raw(n) }
    }

    /// Retrieves `&Dyn` from [`FuseBox`].
    ///
    /// # Panics
    ///
    /// Panics when `n >= len`
    pub fn get(&self, n: usize) -> &Dyn {
        unsafe { &*self.get_raw(n) }
    }

    /// Returns an iterator over `&Dyn` stored in this [`FuseBox`]
    pub fn iter<'f>(&'f self) -> Iter<'f, Dyn, Sz> {
        Iter::new(self)
    }

    /// Returns an iterator over `&mut Dyn` stored in this [`FuseBox`].
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
