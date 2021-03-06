#![allow(unused_macros)]

use crate::internal_prelude::*;

macro_rules! fail {
    ($err:expr) => (
        return Err(From::from($err));
    );

    ($fmt:expr, $($arg:tt)*) => (
        fail!(format!($fmt, $($arg)*))
    );
}

macro_rules! try_ref_clone {
    ($expr:expr) => {
        match $expr {
            Ok(ref val) => val,
            Err(ref err) => return Err(From::from(err.clone())),
        }
    };
}

macro_rules! ensure {
    ($expr:expr, $err:expr) => (
        if !($expr) {
            fail!($err);
        }
    );
    ($expr: expr, $fmt:expr, $($arg:tt)*) => (
        if !($expr) {
            fail!(format!($fmt, $($arg)*));
        }
    );
}

/// Panics if `$expr` is not an Err(err) with err.description() containing `$err`.
#[cfg(test)]
#[allow(unused_macros)]
macro_rules! assert_err {
    ($expr:expr, $err:expr) => {
        match $expr {
            Ok(_) => {
                panic!("assertion failed: not an error in `{}`", stringify!($expr));
            }
            Err(ref value) => {
                let desc = value.description().to_string();
                if !desc.contains($err) {
                    panic!(
                        "assertion failed: error message `{}` doesn't contain `{}` in `{}`",
                        desc,
                        $err,
                        stringify!($expr)
                    );
                }
            }
        }
    };
}

/// Panics if `$expr` is not an Err(err) with err.description() matching regexp `$err`.
#[cfg(test)]
#[allow(unused_macros)]
macro_rules! assert_err_re {
    ($expr:expr, $err:expr) => {
        match $expr {
            Ok(_) => {
                panic!("assertion failed: not an error in `{}`", stringify!($expr));
            }
            Err(ref value) => {
                use regex::Regex;
                let re = Regex::new($err).unwrap();
                let desc = value.description().to_string();
                if !re.is_match(desc.as_ref()) {
                    panic!(
                        "assertion failed: error message `{}` doesn't match `{}` in `{}`",
                        desc,
                        re,
                        stringify!($expr)
                    );
                }
            }
        }
    };
}

/// Run a potentially unsafe expression in a closure synchronized by a global reentrant mutex.
#[macro_export]
macro_rules! h5lock {
    ($expr:expr) => {{
        #[cfg_attr(feature = "cargo-clippy", allow(clippy::redundant_closure))]
        #[allow(unused_unsafe)]
        unsafe {
            $crate::sync::sync(|| $expr)
        }
    }};
}

/// Convert result of HDF5 call to Result; execution is guarded by a global reentrant mutex.
#[macro_export]
macro_rules! h5call {
    ($expr:expr) => {
        h5lock!($crate::error::h5check($expr))
    };
}

/// `h5try!(..)` is equivalent to try!(h5call!(..)).
macro_rules! h5try {
    ($expr:expr) => {
        match h5call!($expr) {
            Ok(value) => value,
            Err(err) => return Err(From::from(err)),
        }
    };
}

pub(crate) trait H5Get: Copy + Default {
    type Func;

    fn h5get(func: Self::Func, id: hid_t) -> Result<Self>;

    #[inline]
    fn h5get_d(func: Self::Func, id: hid_t) -> Self {
        Self::h5get(func, id).unwrap_or_else(|_| Self::default())
    }
}

macro_rules! h5get {
    ($func:ident($id:expr): $ty:ty) => {
        <($ty,) as $crate::macros::H5Get>::h5get($func as _, $id).map(|x| x.0)
    };
    ($func:ident($id:expr): $($ty:ty),+) => {
        <($($ty),+) as $crate::macros::H5Get>::h5get($func as _, $id)
    };
}

macro_rules! h5get_d {
    ($func:ident($id:expr): $ty:ty) => {
        <($ty,) as $crate::macros::H5Get>::h5get_d($func as _, $id).0
    };
    ($func:ident($id:expr): $($ty:ty),+) => {
        <($($ty),+) as $crate::macros::H5Get>::h5get_d($func as _, $id)
    };
}

macro_rules! impl_h5get {
    ($($name:ident: $ty:ident),+) => {
        impl<$($ty),+> H5Get for ($($ty,)+)
        where
            $($ty: Copy + Default),+,
        {
            type Func = unsafe extern "C" fn(hid_t, $(*mut $ty),+) -> herr_t;

            #[inline]
            fn h5get(func: Self::Func, id: hid_t) -> Result<Self> {
                $(let mut $name: $ty = Default::default();)+
                h5call!(func(id, $(&mut $name),+)).map(|_| ($($name,)+))
            }
        }
    };
}

impl_h5get!(a: A);
impl_h5get!(a: A, b: B);
impl_h5get!(a: A, b: B, c: C);
impl_h5get!(a: A, b: B, c: C, d: D);
