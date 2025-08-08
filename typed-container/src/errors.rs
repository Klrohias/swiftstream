use std::{
    any::type_name,
    fmt::{Debug, Display},
    marker::PhantomData,
    sync::PoisonError,
};

use derivative::Derivative;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    LockPoisoned,
    Duplicated,
    NotFound,
    FailDowncast,
    CircularReference,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicated => write!(f, "Item duplicated"),
            Self::LockPoisoned => write!(f, "Lock poisoned"),
            Self::NotFound => write!(f, "Service not found"),
            Self::FailDowncast => write!(f, "Failed to downcast"),
            Self::CircularReference => write!(f, "Found circular reference"),
        }
    }
}

impl<E> From<PoisonError<E>> for ErrorKind {
    fn from(_: PoisonError<E>) -> Self {
        ErrorKind::LockPoisoned
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Error<T> {
    pub kind: ErrorKind,
    #[derivative(Debug = "ignore")]
    _marker: PhantomData<T>,
}

impl<T> Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, type_name::<T>())
    }
}

impl<T> std::error::Error for Error<T> {}

impl<T> From<ErrorKind> for Error<T> {
    fn from(value: ErrorKind) -> Self {
        Self {
            kind: value,
            _marker: PhantomData,
        }
    }
}

impl<T, E> From<PoisonError<E>> for Error<T> {
    fn from(_: PoisonError<E>) -> Self {
        Self {
            kind: ErrorKind::LockPoisoned,
            _marker: PhantomData,
        }
    }
}
