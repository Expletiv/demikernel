// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Imports
//======================================================================================================================

use ::arrayvec::ArrayVec;
use ::demikernel::{
    inetstack::{
        consts::{MAX_BATCH_SIZE_NUM_PACKETS, MAX_HEADER_SIZE},
        protocols::layer1::PhysicalLayer,
    },
    runtime::{
        fail::Fail,
        memory::{DemiBuffer, DemiMemoryAllocator},
        SharedObject,
    },
};
use ::log::trace;
use ::std::ops::{Deref, DerefMut};

//======================================================================================================================
// Structures
//======================================================================================================================

/// Dummy Runtime
pub struct DummyRuntime {
    /// Shared Member Fields
    /// Random Number Generator
    /// Incoming Queue of Packets
    incoming: crossbeam_channel::Receiver<DemiBuffer>,
    /// Outgoing Queue of Packets
    outgoing: crossbeam_channel::Sender<DemiBuffer>,
}

#[derive(Clone)]

/// Shared Dummy Runtime
pub struct SharedDummyRuntime(SharedObject<DummyRuntime>);

//======================================================================================================================
// Associate Functions
//======================================================================================================================

/// Associate Functions for Dummy Runtime
impl SharedDummyRuntime {
    /// Creates a Dummy Runtime.
    pub fn new(
        incoming: crossbeam_channel::Receiver<DemiBuffer>,
        outgoing: crossbeam_channel::Sender<DemiBuffer>,
    ) -> Self {
        Self(SharedObject::new(DummyRuntime { incoming, outgoing }))
    }
}

//======================================================================================================================
// Trait Implementations
//======================================================================================================================

/// Network Runtime Trait Implementation for Dummy Runtime
impl PhysicalLayer for SharedDummyRuntime {
    fn transmit(&mut self, pkts: ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>) -> Result<(), Fail> {
        for pkt in pkts {
            trace!("transmitting pkt: size={:?}", pkt.len());
            // The packet header and body must fit into whatever physical media we're transmitting over.
            // For this test harness, we 2^16 bytes (u16::MAX) as our limit.
            assert!(pkt.len() < u16::MAX as usize);

            if self.outgoing.try_send(pkt).is_err() {
                return Err(Fail::new(
                    libc::EAGAIN,
                    "Could not push outgoing packet to the shared channel",
                ));
            }
        }
        Ok(())
    }

    fn receive(&mut self) -> Result<ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>, Fail> {
        let mut out = ArrayVec::new();
        if let Some(buf) = self.incoming.try_recv().ok() {
            trace!("receiving pkt: size={:?}", buf.len());
            out.push(buf);
        }
        Ok(out)
    }
}

impl DemiMemoryAllocator for SharedDummyRuntime {
    fn allocate_demi_buffer(&self, size: usize) -> Result<DemiBuffer, Fail> {
        Ok(DemiBuffer::new_with_headroom(size as u16, MAX_HEADER_SIZE as u16))
    }
}

impl Deref for SharedDummyRuntime {
    type Target = DummyRuntime;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl DerefMut for SharedDummyRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
