// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Imports
//======================================================================================================================

#[cfg(all(not(feature = "direct-mapping"), debug_assertions))]
use ::rand::{rngs::SmallRng, RngCore, SeedableRng};
#[cfg(not(feature = "direct-mapping"))]
use ::std::collections::HashMap;
use ::std::hash::Hash;
#[cfg(feature = "direct-mapping")]
use ::std::marker::PhantomData;

#[cfg(feature = "direct-mapping")]
pub struct IdMap<E: Eq + Hash + From<u64> + Into<u64> + Copy, I: From<u64> + Into<u64> + Copy> {
    _phantom: PhantomData<(E, I)>,
    current_id: u64,
}

#[cfg(not(feature = "direct-mapping"))]
pub struct IdMap<E: Eq + Hash + From<u64> + Into<u64> + Copy, I: From<u64> + Into<u64> + Copy> {
    /// Map between external and internal ids.
    ids: HashMap<E, I>,
    /// Small random number generator for external ids.
    #[cfg(debug_assertions)]
    rng: SmallRng,
    #[cfg(not(debug_assertions))]
    current_id: u64,
}

#[cfg(feature = "direct-mapping")]
include!("direct_map.rs");
#[cfg(not(feature = "direct-mapping"))]
include!("indirect_map.rs");
