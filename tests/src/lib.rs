#![feature(never_type)]

use hoard::prelude::*;
use hoard::marshal::*;
use hoard::marshal::blob::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Outpoint {
    txid: [u8;32],
    n: Le<u32>,
}

/*
#[repr(C)]
#[derive(Debug)]
pub struct TxOut<Z: Zone> {
    value: Le<u64>,
    script: OwnedPtr<[u8], Z>,
}

impl<Y: Zone, Z: Zone> Encoded<Y> for TxOut<Z> {
    type Encoded = TxOut<Y>;
}

impl<'a, Y: Zone, Z: Zone> Encode<'a, Y> for TxOut<Z>
where Z: Encode<'a, Y>
{
    type State = (<Le<u64> as Encode<'a, Y>>::State,
                  <OwnedPtr<[u8], Z> as Encode<'a, Y>>::State);

    fn save_children(&'a self) -> Self::State {
        (Encode::<'a,Y>::save_children(&self.value),
         Encode::<'a,Y>::save_children(&self.script))
    }

    fn poll<D: Dumper<Y>>(&self, state: &mut Self::State, dumper: D) -> Result<D, D::Error> {
        let dumper = Encode::poll(&self.value, &mut state.0, dumper)?;
        let dumper = Encode::poll(&self.script, &mut state.1, dumper)?;
        Ok(dumper)
    }

    fn encode_blob<W: WriteBlob>(&self, state: &Self::State, dst: W) -> Result<W::Ok, W::Error> {
        dst.write::<Y,_>(&self.value, &state.0)?
           .write::<Y,_>(&self.script, &state.1)?
           .finish()
    }
}

impl<Z: Zone> Validate for TxOut<Z> {
    type Error = <OwnedPtr<[u8], Z> as Validate>::Error;

    #[inline(always)]
    fn validate<B: BlobValidator<Self>>(blob: B) -> Result<B::Ok, B::Error> {
        let mut blob = blob.validate_struct();
        blob.field::<Le<u64>,_>(|x| match x {})?;
        blob.field::<OwnedPtr<[u8], Z>,_>(Into::into)?;
        unsafe { blob.assume_valid() }
    }
}

unsafe impl<Z: Zone> Load<Z> for TxOut<Z> {
    type ValidateChildren = (<Le<u64> as Load<Z>>::ValidateChildren, <OwnedPtr<[u8], Z> as Load<Z>>::ValidateChildren);

    #[inline(always)]
    fn validate_children(&self) -> Self::ValidateChildren {
        (Load::<Z>::validate_children(&self.value),
         Load::<Z>::validate_children(&self.script))
    }
}
*/
