use std::alloc::{GlobalAlloc, System, Layout};
use std::borrow::Borrow;
use std::cmp;
use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop};
use std::num::NonZeroU64;
use std::ptr::NonNull;

use thiserror::Error;
use leint::Le;

use owned::{IntoOwned, Take};

use crate::pointee::Pointee;
use crate::blob::*;
use crate::load::*;
use crate::save::*;
use crate::primitive::*;
use crate::ptr::*;

use crate::heap::HeapPtr;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Offset<'pile, 'version> {
    marker: PhantomData<(
                fn(&'pile ()) -> &'pile (),
                &'version (),
            )>,
    pub(super) raw: Le<NonZeroU64>,
}

impl fmt::Debug for Offset<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.get().fmt(f)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct OffsetMut<'p, 'v, A = System> {
    marker: PhantomData<A>,
    inner: Offset<'p, 'v>,
}

impl fmt::Debug for OffsetMut<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind().fmt(f)
    }
}

unsafe impl<'p, 'v> Persist for Offset<'p, 'v> {}
unsafe impl<'p, 'v, A> Persist for OffsetMut<'p, 'v, A> {}

impl<'p, 'v, A> Borrow<OffsetMut<'p, 'v, A>> for Offset<'p, 'v> {
    fn borrow(&self) -> &OffsetMut<'p, 'v, A> {
        self.as_ref()
    }
}

impl<'p, 'v, A> AsRef<OffsetMut<'p, 'v, A>> for Offset<'p, 'v> {
    fn as_ref(&self) -> &OffsetMut<'p, 'v, A> {
        // SAFETY: #[repr(transparent)]
        unsafe { &*(self as *const Self as *const _) }
    }
}

impl<'p, 'v> From<Offset<'p, 'v>> for usize {
    #[inline(always)]
    fn from(offset: Offset<'p, 'v>) -> usize {
        offset.get()
    }
}

impl<'p, 'v> From<Offset<'p, 'v>> for OffsetMut<'p, 'v> {
    fn from(inner: Offset<'p, 'v>) -> Self {
        Self {
            marker: PhantomData,
            inner,
        }
    }
}

impl cmp::PartialEq<usize> for Offset<'_, '_> {
    fn eq(&self, other: &usize) -> bool {
        self.get() == *other
    }
}
impl cmp::PartialEq<Offset<'_, '_>> for usize {
    fn eq(&self, other: &Offset<'_, '_>) -> bool {
        *self == other.get()
    }
}

impl<'p, 'v> Offset<'p, 'v> {
    pub const MAX: usize = (1 << 62) - 1;

    #[inline(always)]
    pub fn new(offset: usize) -> Option<Self> {
        let offset = offset as u64;
        offset.checked_shl(1).map(|offset|
            Self {
                marker: PhantomData,
                raw: NonZeroU64::new(offset | 1).unwrap().into(),
            }
        )
    }

    #[inline(always)]
    pub fn cast<'p2, 'v2>(&self) -> Offset<'p2, 'v2> {
        Offset {
            marker: PhantomData,
            raw: self.raw,
        }
    }

    #[inline(always)]
    pub fn get(&self) -> usize {
        (self.raw.get().get() >> 1) as usize
    }

    #[inline(always)]
    pub fn dangling() -> Self {
        Self::new(Self::MAX).unwrap()
    }

    pub fn to_static(&self) -> Offset<'static, 'static> {
        Offset {
            marker: PhantomData,
            raw: self.raw,
        }
    }
}

#[derive(Debug, Error)]
#[error("FIXME")]
pub struct ValidateBlobOffsetError;

impl<'p, 'v> ValidateBlob for Offset<'p, 'v> {
    const BLOB_LEN: usize = mem::size_of::<Self>();
    type Error = ValidateBlobOffsetError;

    fn validate_blob<'a>(blob: BlobValidator<'a, Self>) -> Result<ValidBlob<'a, Self>, Self::Error> {
        todo!()
    }
}

impl<'p, 'v, A> ValidateBlob for OffsetMut<'p, 'v, A> {
    const BLOB_LEN: usize = mem::size_of::<Self>();
    type Error = ValidateBlobOffsetError;

    fn validate_blob<'a>(blob: BlobValidator<'a, Self>) -> Result<ValidBlob<'a, Self>, Self::Error> {
        todo!()
    }
}

impl<'p, 'v, Z> Decode<Z> for Offset<'p, 'v> {
    fn decode_blob(blob: BlobDecoder<Z, Self>) -> Self {
        todo!()
    }
}

impl<'p, 'v, Z> Decode<Z> for OffsetMut<'p, 'v> {
    fn decode_blob(blob: BlobDecoder<Z, Self>) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub enum Kind<'p, 'v> {
    Offset(Offset<'p, 'v>),
    Ptr(NonNull<u16>),
}

impl<'p, 'v, A> OffsetMut<'p, 'v, A> {
    #[inline]
    pub unsafe fn from_ptr(ptr: NonNull<u16>) -> Self {
        let raw = ptr.as_ptr() as usize as u64;

        debug_assert_eq!(raw & 1, 0,
                   "{:p} unaligned", ptr);

        mem::transmute(ptr.as_ptr() as usize as u64)
    }

    #[inline]
    pub fn kind(&self) -> Kind<'p, 'v> {
        if self.inner.raw.get().get() & 1 == 1 {
            Kind::Offset(self.inner)
        } else {
            Kind::Ptr(unsafe {
                let raw = self.inner.raw.get().get();
                NonNull::new_unchecked(raw as usize as *mut u16)
            })
        }
    }

    #[inline(always)]
    pub fn get_offset(&self) -> Option<Offset<'p, 'v>> {
        match self.kind() {
            Kind::Offset(offset) => Some(offset),
            Kind::Ptr(_) => None,
        }
    }

    #[inline(always)]
    pub fn get_ptr(&self) -> Option<NonNull<u16>> {
        match self.kind() {
            Kind::Ptr(ptr) => Some(ptr),
            Kind::Offset(_) => None,
        }
    }
}

impl<'p, 'v> Ptr for Offset<'p, 'v> {
    type Persist = Offset<'static, 'static>;

    unsafe fn dealloc<T: ?Sized + Pointee>(&self, _: T::Metadata) {
        // nothing to do here
    }

    fn duplicate(&self) -> Self {
        Self {
            marker: PhantomData,
            raw: self.raw,
        }
    }

    unsafe fn clone_unchecked_with<T: ?Sized + Pointee, U, F>(&self, metadata: T::Metadata, _: F) -> Own<T, Self> {
        Own::new_unchecked(Fat::new(*self, metadata))
    }

    unsafe fn try_get_dirty_unchecked<T: ?Sized + Pointee>(&self, _: T::Metadata) -> Result<&T, Self::Persist> {
        Err(self.cast())
    }

    unsafe fn try_take_dirty_unchecked<T: ?Sized + Pointee>(self, _: T::Metadata) -> Result<T::Owned, Self::Persist>
        where T: IntoOwned
    {
        Err(self.cast())
    }
}

impl<'p, 'v> Ptr for OffsetMut<'p, 'v> {
    type Persist = Offset<'static, 'static>;

    unsafe fn dealloc<T: ?Sized + Pointee>(&self, metadata: T::Metadata) {
        match self.kind() {
            Kind::Offset(_) => {},
            Kind::Ptr(ptr) => HeapPtr(ptr).dealloc::<T>(metadata),
        }
    }

    fn alloc<T: ?Sized + Pointee, U: Take<T>>(src: U) -> Own<T, Self> {
        let fat = HeapPtr::alloc(src).into_inner();

        unsafe {
            Own::new_unchecked(Fat::new(Self::from_ptr(fat.raw.0), fat.metadata))
        }
    }

    fn duplicate(&self) -> Self {
        Self {
            marker: PhantomData,
            inner: self.inner.duplicate(),
        }
    }

    unsafe fn clone_unchecked_with<T: ?Sized, U, F>(&self, metadata: T::Metadata, f: F) -> Own<T, Self>
        where T: Pointee,
              F: FnOnce(&T) -> U,
              U: Take<T>,
    {
        match self.try_get_dirty_unchecked::<T>(metadata) {
            Err(offset) => {
                Own::new_unchecked(Fat::new(
                        Self {
                            marker: PhantomData,
                            inner: offset.cast(),
                        },
                        metadata
                ))
            },
            Ok(value) => Self::alloc(f(value)),
        }
    }

    unsafe fn try_get_dirty_unchecked<T: ?Sized + Pointee>(&self, metadata: T::Metadata) -> Result<&T, Self::Persist> {
        match self.kind() {
            Kind::Ptr(ptr) => Ok(&*T::make_fat_ptr_mut(ptr.cast().as_ptr(), metadata)),
            Kind::Offset(offset) => Err(offset.cast()),
        }
    }

    unsafe fn try_take_dirty_unchecked<T: ?Sized + Pointee>(self, metadata: T::Metadata) -> Result<T::Owned, Self::Persist>
        where T: IntoOwned
    {
        match self.kind() {
            Kind::Offset(offset) => Err(offset.cast()),
            Kind::Ptr(ptr) => {
                Ok(crate::heap::HeapPtr(ptr).try_take_dirty_unchecked::<T>(metadata).into_ok())
            },
        }
    }
}

impl<'p,'v> Default for OffsetMut<'p, 'v> {
    fn default() -> Self {
        Offset::dangling().into()
    }
}

impl<R> Encoded<R> for Offset<'_, '_> {
    type Encoded = Self;
}

impl<R> Encoded<R> for OffsetMut<'_, '_> {
    type Encoded = Self;
}

impl<Q, R> Encode<'_, Q, R> for Offset<'_, '_> {
    type State = ();
    fn init_encode_state(&self) -> () {}
    fn encode_poll<D>(&self, _: &mut (), dst: D) -> Result<D, D::Error>
        where D: Dumper<Source=Q, Target=R>
    {
        Ok(dst)
    }
    fn encode_blob<W: WriteBlob>(&self, _: &(), dst: W) -> Result<W::Done, W::Error> {
        dst.write_primitive(&self.raw)?
           .done()
    }
}

impl Primitive for Offset<'_, '_> {}

#[derive(Debug, Default)]
pub struct ShallowDumper<'p, 'v> {
    marker: PhantomData<OffsetMut<'p, 'v>>,
    written: Vec<u8>,
    initial_offset: usize,
}

impl<'p, 'v> Dumper for ShallowDumper<'p, 'v> {
    type Source = OffsetMut<'p, 'v>;
    type Target = Offset<'p, 'v>;

    type Error = !;
    unsafe fn try_save_ptr<'a, T: ?Sized>(&mut self, ptr: &'a Self::Source, metadata: T::Metadata)
        -> Result<Self::Target, &'a T>
    where T: Pointee
    {
        match ptr.try_get_dirty_unchecked::<T>(metadata) {
            Ok(r) => Err(r),
            Err(offset) => Ok(offset.cast())
        }
    }

    fn save_ptr<'a, T: ?Sized>(mut self, value: &'a T, state: &T::State) -> Result<(Self, Self::Target), Self::Error>
        where T: Save<'a, Self::Source, Self::Target>
    {
        let offset = self.initial_offset
                         .checked_add(self.written.len())
                         .and_then(Offset::new)
                         .expect("overflow");

        let written = mem::replace(&mut self.written, vec![]);
        self.written = value.save_blob(state, written).into_ok();
        Ok((self, offset))
    }
}

impl<'p, 'v> ShallowDumper<'p, 'v> {
    pub fn new(initial_offset: usize) -> Self {
        Self {
            marker: PhantomData,
            written: vec![],
            initial_offset,
        }
    }

    pub fn save<'a, T>(self, value: &'a T) -> (Vec<u8>, Offset<'p, 'v>)
        where T: Save<'a, OffsetMut<'p, 'v>, Offset<'p, 'v>>
    {
        let mut state = value.init_save_state();
        let this = value.save_poll(&mut state, self).into_ok();
        let (this, offset) = this.save_ptr(value, &state).into_ok();
        (this.written, offset)
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pile::Pile;
    use crate::bag::Bag;

    #[test]
    fn test_shallow_dumper() {
        let (buf, offset) = ShallowDumper::new(0).save(&42u8);
        assert_eq!(offset, 0);
        assert_eq!(buf, &[42]);

        let pile = Pile::default();
        let bag = Bag::new_in(42u8, pile);

        let (buf, offset) = ShallowDumper::new(0).save(&bag);
        assert_eq!(offset, 1);
        assert_eq!(buf, &[42, 1,0,0,0,0,0,0,0]);

        let bag2 = Bag::new_in(bag, pile);
        let (buf, offset) = ShallowDumper::new(0).save(&bag2);
        assert_eq!(offset, 9);
        assert_eq!(buf,
            &[42,
              1,0,0,0,0,0,0,0,
              3,0,0,0,0,0,0,0,
            ]);
    }
}
*/
