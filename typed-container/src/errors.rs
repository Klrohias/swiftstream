use std::{error::Error, fmt::Display, sync::PoisonError};

use smol_str::SmolStr;

#[derive(Debug)]
pub enum RegisterError {
    LockPoisoned,
    Duplicated(SmolStr),
}

impl Display for RegisterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicated(type_name) => write!(f, "Item duplicated: {}", type_name),
            Self::LockPoisoned => write!(f, "Lock poisoned"),
        }
    }
}

impl Error for RegisterError {}

impl<T> From<PoisonError<T>> for RegisterError {
    fn from(_: PoisonError<T>) -> Self {
        Self::LockPoisoned
    }
}

#[derive(Debug)]
pub enum GetServiceError {
    LockPoisoned,
    NotFound(SmolStr),
    FailDowncast(SmolStr),
    FailConstruct(SmolStr),
    FailRegister(RegisterError),
}

impl Display for GetServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(type_name) => write!(f, "Service not found: {}", type_name),
            Self::LockPoisoned => write!(f, "Lock poisoned"),
            Self::FailDowncast(type_name) => write!(f, "Failed to downcast: {}", type_name),
            Self::FailConstruct(type_name) => write!(f, "Failed to construct: {}", type_name),
            Self::FailRegister(e) => e.fmt(f),
        }
    }
}

impl Error for GetServiceError {}

impl<T> From<PoisonError<T>> for GetServiceError {
    fn from(_: PoisonError<T>) -> Self {
        Self::LockPoisoned
    }
}

impl From<RegisterError> for GetServiceError {
    fn from(e: RegisterError) -> Self {
        Self::FailRegister(e)
    }
}
