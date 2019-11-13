use super::*;

use core::slice;
use core::mem;

use leint::Le;

impl<Z: Zone> Save<Z> for ! {
    const BLOB_LAYOUT: BlobLayout = BlobLayout::never();

    type SavePoll = Self;

    fn save_poll(this: impl Take<Self>) -> Self::SavePoll {
        match this.take_sized() {}
    }
}

impl<Z: Zone> SavePoll<Z> for ! {
    type Target = !;

    fn encode_blob<W: WriteBlob>(&self, _dst: W) -> Result<W::Done, W::Error> {
        match *self {}
    }
}

impl<Z: Zone> Load<Z> for ! {
    type Error = !;

    type ValidateChildren = ();
    fn validate_blob<'p>(_blob: Blob<'p, Self, Z>) -> Result<ValidateBlob<'p, Self, Z>, Self::Error> {
        panic!()
    }

    fn load_blob<'p>(_: FullyValidBlob<'p, Self, Z>, _: &impl Loader<Z>) -> Ref<'p, Self> {
        panic!()
    }

    fn decode_blob<'p>(_: FullyValidBlob<'p, Self, Z>, _: &impl Loader<Z>) -> Self {
        panic!()
    }

    fn deref_blob<'p>(_: FullyValidBlob<'p, Self, Z>) -> &'p Self {
        panic!()
    }
}

unsafe impl Persist for ! {}

impl<Z: Zone> Save<Z> for () {
    const BLOB_LAYOUT: BlobLayout = BlobLayout::new(mem::size_of::<Self>());

    type SavePoll = Self;

    fn save_poll(this: impl Take<Self>) -> Self::SavePoll {
        this.take_sized()
    }
}

impl<Z: Zone> SavePoll<Z> for () {
    type Target = ();

    fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Done, W::Error> {
        dst.done()
    }
}

impl<Z: Zone> Load<Z> for () {
    type Error = !;

    type ValidateChildren = ();
    fn validate_blob<'p>(blob: Blob<'p, Self, Z>) -> Result<ValidateBlob<'p, Self, Z>, Self::Error> {
        Ok(blob.assume_valid(()))
    }

    fn load_blob<'p>(blob: FullyValidBlob<'p, Self, Z>, _: &impl Loader<Z>) -> Ref<'p, Self> {
        unsafe { blob.assume_valid_ref() }
    }

    fn decode_blob<'p>(blob: FullyValidBlob<'p, Self, Z>, _: &impl Loader<Z>) -> Self {
        unsafe { *blob.assume_valid() }
    }
}

unsafe impl Persist for () {}


macro_rules! impl_aligned_ints {
    ($( $t:ty, )+) => {
        $(
            impl<Z: Zone> Save<Z> for $t {
                const BLOB_LAYOUT: BlobLayout = BlobLayout::new(mem::size_of::<Self>());
                type SavePoll = Self;

                fn save_poll(this: impl Take<Self>) -> Self::SavePoll {
                    this.take_sized()
                }
            }

            impl<Z: Zone> SavePoll<Z> for $t {
                type Target = $t;

                fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Done, W::Error> {
                    dst.write_bytes(&self.to_le_bytes())?
                       .done()
                }
            }

            impl<Z: Zone> Load<Z> for $t {
                type Error = !;

                fn validate_blob<'p>(blob: Blob<'p, Self, Z>) -> Result<ValidateBlob<'p, Self, Z>, Self::Error> {
                    Ok(blob.assume_valid(()))
                }

                type ValidateChildren = ();

                fn decode_blob<'p>(blob: FullyValidBlob<'p, Self, Z>, _: &impl Loader<Z>) -> Self {
                    let mut r = [0; mem::size_of::<Self>()];
                    r.copy_from_slice(&blob[..]);
                    <$t>::from_le_bytes(r)
                }
            }
        )+
    }
}

impl_aligned_ints! {
    u16, u32, u64, u128,
    i16, i32, i64, i128,
}

macro_rules! unsafe_impl_persist_ints {
    ($( $t:ty, )+) => {
        $(
            impl<Z: Zone> Save<Z> for $t {
                const BLOB_LAYOUT: BlobLayout = BlobLayout::new(mem::size_of::<Self>());
                type SavePoll = Self;

                fn save_poll(this: impl Take<Self>) -> Self::SavePoll {
                    this.take_sized()
                }
            }

            impl<Z: Zone> SavePoll<Z> for $t {
                type Target = $t;

                fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Done, W::Error> {
                    let src = unsafe { slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<Self>()) };
                    dst.write_bytes(src)?
                       .done()
                }
            }

            impl<Z: Zone> Load<Z> for $t {
                type Error = !;

                fn validate_blob<'p>(blob: Blob<'p, Self, Z>) -> Result<ValidateBlob<'p, Self, Z>, Self::Error> {
                    Ok(blob.assume_valid(()))
                }

                type ValidateChildren = ();

                fn load_blob<'p>(blob: FullyValidBlob<'p, Self, Z>, _: &impl Loader<Z>) -> Ref<'p, Self> {
                    unsafe { blob.assume_valid_ref() }
                }

                fn decode_blob<'p>(blob: FullyValidBlob<'p, Self, Z>, _: &impl Loader<Z>) -> Self {
                    unsafe { *blob.assume_valid() }
                }
            }

            unsafe impl Persist for $t {}
        )+
    }
}

unsafe_impl_persist_ints! {
    u8, Le<u16>, Le<u32>, Le<u64>, Le<u128>,
    i8, Le<i16>, Le<i32>, Le<i64>, Le<i128>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(encode(42u8),
                   &[42]);
    }
}