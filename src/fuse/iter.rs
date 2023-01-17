use std::{
    intrinsics::{exact_div, unchecked_sub},
    marker::PhantomData,
    mem::size_of,
    ptr::{self, NonNull},
};

use super::{FuseBox, Header};

macro_rules! is_empty {
    ($self:ident) => {
        $self.headers_ptr.as_ptr() as *const _ == $self.headers_end
    };
}

macro_rules! len {
    ($self:ident) => {{
        let start = $self.headers_ptr.as_ptr() as usize;
        let size = size_of::<Header<Dyn>>();

        let diff = unsafe { unchecked_sub($self.headers_end as usize, start) };
        unsafe { exact_div(diff, size) }
    }};
}

macro_rules! impl_iter {
    ($iter:tt $(, $mut:tt)?) => {
        pub struct $iter<'f, Dyn>
        where
            Dyn: ?Sized,
        {
            headers_ptr: NonNull<Header<Dyn>>,
            headers_end: *const Header<Dyn>,
            data_base_ptr: NonNull<u8>,
            _tag: PhantomData<&'f $($mut)? FuseBox<Dyn>>,
        }

        impl<'f, Dyn> $iter<'f, Dyn>
        where
            Dyn: ?Sized,
        {
            pub(crate) fn new(fused: &'f $($mut)? FuseBox<Dyn>) -> Self {
                let headers_ptr =
                    unsafe { NonNull::new_unchecked(fused.headers.as_ptr() as *mut _) };
                let headers_end = unsafe { fused.headers.as_ptr().add(fused.headers.len()) };
                let data_base_ptr = fused.inner;
                Self {
                    headers_ptr,
                    headers_end,
                    data_base_ptr,
                    _tag: Default::default(),
                }
            }
        }

        impl<'f, Dyn> Iterator for $iter<'f, Dyn>
        where
            Dyn: ?Sized,
        {
            type Item = &'f $($mut)? Dyn;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if is_empty!(self) {
                    return None;
                }
                unsafe {
                    let next_ptr = self.headers_ptr.as_ptr();
                    let Header { offset, meta } = *next_ptr;

                    let ptr = self.data_base_ptr.as_ptr().add(offset).cast();

                    self.headers_ptr = NonNull::new_unchecked(next_ptr.add(1));

                    Some(&$($mut)? *ptr::from_raw_parts_mut::<Dyn>(ptr, meta))
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = len!(self);
                (len, Some(len))
            }

            #[inline]
            fn count(self) -> usize
            where
                Self: Sized,
            {
                len!(self)
            }

            #[inline]
            fn last(self) -> Option<Self::Item>
            where
                Self: Sized,
            {
                if is_empty!(self) {
                    return None;
                }
                unsafe {
                    let Header { offset, meta } = *self.headers_end.sub(1);

                    let ptr = self.data_base_ptr.as_ptr().add(offset).cast();
                    Some(& $($mut)? *ptr::from_raw_parts_mut::<Dyn>(ptr, meta))
                }
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                if n >= len!(self) {
                    return None;
                }
                unsafe {
                    let next_ptr = self.headers_ptr.as_ptr().add(n);
                    let Header { offset, meta } = *next_ptr;

                    let ptr = self.data_base_ptr.as_ptr().add(offset).cast();

                    self.headers_ptr = NonNull::new_unchecked(next_ptr);

                    Some(& $($mut)? *ptr::from_raw_parts_mut::<Dyn>(ptr, meta))
                }
            }
        }

        impl<'f, Dyn> ExactSizeIterator for $iter<'f, Dyn>
        where
            Dyn: ?Sized,
        {
            #[inline]
            fn len(&self) -> usize {
                len!(self)
            }
        }
    };
}

impl_iter!(Iter);
impl_iter!(IterMut, mut);
