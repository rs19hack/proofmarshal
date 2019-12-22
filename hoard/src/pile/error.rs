use core::ptr::NonNull;

use std::backtrace::Backtrace;

use super::*;

use crate::load::{Validate, ValidationError};

/// An attempt to dereference a pile offset failed.
#[derive(Debug, PartialEq, Eq)]
pub struct OffsetError<'p,'v> {
    pile: Pile<'p, 'v>,
    pub(crate) offset: Offset<'p, 'v>,
}

impl<'p,'v> OffsetError<'p,'v> {
    pub fn new<T: ?Sized + Validate>(pile: &Pile<'p,'v>, ptr: &FatPtr<T, Pile<'p,'v>>) -> Self {
        Self {
            pile: *pile,
            offset: ptr.raw,
        }
    }
}

#[derive(Debug)]
pub enum DerefError<'p, 'v, E = Box<dyn ValidationError>> {
    Offset(OffsetError<'p, 'v>),
    Value {
        pile: Pile<'p, 'v>,
        offset: Offset<'p, 'v>,
        err: E,
    }
}

pub enum ValidatorError<'p, 'v> {
    Offset {
        offset: Offset<'p,'v>,
    },
    Value {
        offset: Offset<'p, 'v>,
        err: Box<dyn ValidationError>,
    },
    Padding {
        offset: Offset<'p, 'v>,
    },
}

impl<'p,'v, E> From<OffsetError<'p, 'v>> for DerefError<'p,'v, E> {
    fn from(err: OffsetError<'p, 'v>) -> Self {
        DerefError::Offset(err)
    }
}

impl From<DerefError<'_, '_>> for DerefErrorPayload {
    fn from(err: DerefError) -> DerefErrorPayload {
        match err {
            DerefError::Offset(err) => err.into(),
            DerefError::Value { pile, offset, err } => {
                Self::Value {
                    mapping: NonNull::from(pile.slice()).cast(),
                    offset: offset.to_static(),
                    err,
                }
            },
        }
    }
}

#[derive(Debug)]
pub(crate) enum DerefErrorPayload {
    Offset {
        mapping: NonNull<NonNull<[u8]>>,
        offset: Offset<'static, 'static>,
    },
    Value {
        mapping: NonNull<NonNull<[u8]>>,
        offset: Offset<'static, 'static>,
        err: Box<dyn ValidationError>,
    }
}

unsafe impl Send for DerefErrorPayload {}

impl From<OffsetError<'_, '_>> for DerefErrorPayload {
    fn from(err: OffsetError<'_, '_>) -> Self {
        DerefErrorPayload::Offset {
            mapping: NonNull::from(err.pile.slice()).cast(),
            offset: err.offset.to_static(),
        }
    }
}
