// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Imports
//======================================================================================================================

use ::arrayvec::ArrayVec;

use crate::{
    inetstack::consts::MAX_BATCH_SIZE_NUM_PACKETS,
    inetstack::protocols::layer4::ephemeral::EphemeralPorts,
    runtime::{
        fail::Fail,
        memory::{DemiBuffer, DemiMemoryAllocator},
    },
};
pub use ::std::any::Any;

//======================================================================================================================
// Traits
//======================================================================================================================

/// API for the Physical Layer for any underlying hardware that implements a raw NIC interface (e.g., DPDK, raw
/// sockets). It must implement [DemiMemoryAllocator] to specify how to allocate DemiBuffers for the physical layer.
pub trait PhysicalLayer: 'static + DemiMemoryAllocator {
    /// Transmits a batch of [DemiBuffer].
    fn transmit(&mut self, pkts: ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>) -> Result<(), Fail>;

    /// Receives a batch of [DemiBuffer].
    fn receive(&mut self) -> Result<ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>, Fail>;

    /// Returns the ephemeral ports on which this physical layer may operate. If none, any valid ephemeral port may be
    /// used.
    fn ephemeral_ports(&self) -> EphemeralPorts {
        EphemeralPorts::default()
    }
}
