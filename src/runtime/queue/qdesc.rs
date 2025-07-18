// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Structures
//======================================================================================================================

/// IO Queue Descriptor
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct QDesc(u32);

//======================================================================================================================
// Associated Functions
//======================================================================================================================

impl QDesc {
    pub const MAX: u32 = u32::MAX;
}

//======================================================================================================================
// Trait Implementations
//======================================================================================================================

impl From<QDesc> for i32 {
    fn from(val: QDesc) -> Self {
        val.0 as i32
    }
}

impl From<i32> for QDesc {
    fn from(val: i32) -> Self {
        QDesc(val as u32)
    }
}

impl From<QDesc> for u32 {
    fn from(val: QDesc) -> Self {
        val.0
    }
}

impl From<u32> for QDesc {
    fn from(val: u32) -> Self {
        QDesc(val)
    }
}
