use core::mem;

pub unsafe trait CastRef<T: ?Sized> {
    fn as_cast_ref(&self) -> &T;
}

pub unsafe trait CastMut<T: ?Sized> : CastRef<T> {
    fn as_cast_mut(&mut self) -> &mut T;
}

pub unsafe trait Cast<T> : CastRef<T> {
    fn cast(self) -> T
        where Self: Sized;
}


pub unsafe trait TryCastRef<T: ?Sized> {
    type Error;
    fn try_cast_ref(&self) -> Result<&T, Self::Error>;
}

pub unsafe trait TryCastMut<T: ?Sized> : TryCastRef<T> {
    fn try_cast_mut(&mut self) -> Result<&mut T, Self::Error>;
}

pub unsafe trait TryCast<T> : TryCastRef<T> {
    fn try_cast(self) -> Result<T, Self::Error> where Self: Sized {
        assert_eq!(mem::size_of::<Self>(), mem::size_of::<T>());
        assert_eq!(mem::align_of::<Self>(), mem::align_of::<T>());

        match self.try_cast_ref() {
            Err(e) => Err(e),
            Ok(_) => {
                let this = mem::ManuallyDrop::new(self);
                unsafe { mem::transmute_copy(&this) }
            }
        }
    }
}

unsafe impl<T: ?Sized, U: ?Sized> CastRef<U> for T
where T: TryCastRef<U>,
      T::Error: Into<!>
{
    fn as_cast_ref(&self) -> &U {
        match self.try_cast_ref() {
            Ok(r) => r,
            Err(e) => e.into(),
        }
    }
}

unsafe impl<T: ?Sized, U: ?Sized> CastMut<U> for T
where T: TryCastMut<U>,
      T::Error: Into<!>
{
    fn as_cast_mut(&mut self) -> &mut U {
        match self.try_cast_mut() {
            Ok(r) => r,
            Err(e) => e.into(),
        }
    }
}

unsafe impl<T: ?Sized, U> Cast<U> for T
where T: TryCast<U>,
      T::Error: Into<!>
{
    fn cast(self) -> U where Self: Sized {
        match self.try_cast() {
            Ok(r) => r,
            Err(e) => e.into(),
        }
    }
}