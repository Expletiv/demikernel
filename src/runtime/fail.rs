// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Imports
//======================================================================================================================

use ::libc::{c_int, EIO};
use ::std::{error, fmt, io};

//======================================================================================================================
// Structures
//======================================================================================================================

#[derive(Clone)]
pub struct Fail {
    pub errno: c_int,
    pub cause: String,
}

//======================================================================================================================
// Associate Functions
//======================================================================================================================

impl Fail {
    pub fn new(errno: i32, cause: &str) -> Self {
        Self {
            errno,
            cause: cause.to_string(),
        }
    }
}

//======================================================================================================================
// Trait Implementations
//======================================================================================================================

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {:?}: {:?}", self.errno, self.cause)
    }
}

impl fmt::Debug for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {:?}: {:?}", self.errno, self.cause)
    }
}

impl error::Error for Fail {}

impl From<io::Error> for Fail {
    fn from(_: io::Error) -> Self {
        Self {
            errno: EIO,
            cause: "I/O error".to_string(),
        }
    }
}

impl From<std::num::TryFromIntError> for Fail {
    fn from(_: std::num::TryFromIntError) -> Self {
        Fail::new(libc::ERANGE, "integer conversion error")
    }
}
