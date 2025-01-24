// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

//======================================================================================================================
// Imports
//======================================================================================================================

//! The "direct-mapping" feature is used to expose internal ids directly to the user. This is useful for debugging and
//! testing purposes. BEWARE: Turning on this feature will have a performance impact.

use ::std::hash::Hash;

#[cfg(not(feature = "direct-mapping"))]
use ::rand::{rngs::SmallRng, RngCore, SeedableRng};
#[cfg(not(feature = "direct-mapping"))]
use ::std::collections::HashMap;

#[cfg(feature = "direct-mapping")]
use std::marker::PhantomData;

//======================================================================================================================
// Constants
//======================================================================================================================

/// Performance note: These two flags were benchmarked with the scheduler insert benchmark on a
/// release build and have the following impact on performance. The number is the average of 5 runs of the test and
/// performed on an Azure Standard D16ds v4 (16 vcpus, 64 GiB memory) VM running Linux (ubuntu 22.04).
/// Direct vs indirect mapping: 152 for direct, indirect is below.
/// Randomize vs non random: 332ns vs 154ns

/// This flag controls how the ids are allocated, either randomly or in a Fibonacci sequence.
#[cfg(not(feature = "direct-mapping"))]
#[cfg(debug_assertions)]
const RANDOMIZE: bool = true;
#[cfg(not(feature = "direct-mapping"))]
#[cfg(not(debug_assertions))]
const RANDOMIZE: bool = false;

/// Arbitrary size chosen to pre-allocate the hashmap. This improves performance by 6ns on average on our scheduler
/// insert benchmark.
#[cfg(not(feature = "direct-mapping"))]
const DEFAULT_SIZE: usize = 1024;

/// Seed for the random number generator used to generate tokens.
/// This value was chosen arbitrarily.
#[cfg(not(feature = "direct-mapping"))]
const SCHEDULER_SEED: u64 = 42;
#[cfg(not(feature = "direct-mapping"))]
const MAX_RETRIES_ID_ALLOC: usize = 500;

//======================================================================================================================
// Structures
//======================================================================================================================

/// This data structure is a general-purpose map for obfuscating ids from external modules. It takes an external id type
/// and an internal id type and translates between the two. The ID types must be basic types that can be converted back
/// and forth between u64 and therefore each other.
#[cfg(not(feature = "direct-mapping"))]
#[derive(Debug)]
pub struct IdMap<E: Eq + Hash + From<u64> + Into<u64> + Copy, I: From<u64> + Into<u64> + Copy> {
    /// Map between external and internal ids.
    ids: HashMap<E, I>,
    /// Small random number generator for external ids.
    rng: SmallRng,
    /// For non-random id generation, we keep the last 2 id numbers for a Fibonacci calculation.
    last_id: u64,
    current_id: u64,
}

#[cfg(feature = "direct-mapping")]
#[derive(Debug)]
pub struct IdMap<E: Eq + Hash + From<u64> + Into<u64> + Copy, I: From<u64> + Into<u64> + Copy> {
    _phantom: PhantomData<(E, I)>,
}

//======================================================================================================================
// Associate Functions
//======================================================================================================================

#[cfg(not(feature = "direct-mapping"))]
impl<E: Eq + Hash + From<u64> + Into<u64> + Copy, I: From<u64> + Into<u64> + Copy> IdMap<E, I> {
    pub fn get(&self, external_id: &E) -> Option<I> {
        self.ids.get(external_id).copied()
    }

    #[allow(dead_code)]
    /// Insert a mapping between a specified external and internal id.
    pub fn insert(&mut self, external_id: E, internal_id: I) -> Option<I> {
        self.ids.insert(external_id, internal_id)
    }

    /// Remove a mapping between a specificed external and internal id. If the mapping exists, then return the internal
    /// id mapped to the external id.
    pub fn remove(&mut self, external_id: &E) -> Option<I> {
        self.ids.remove(external_id)
    }

    /// Generate a new id and insert the mapping to the internal id. If the id is currently in use, keep generating
    /// until we find an unused id (up to a maximum number of tries).
    pub fn insert_with_new_id(&mut self, internal_id: I) -> E {
        if RANDOMIZE {
            // Otherwise, allocate a new external id.
            for _ in 0..MAX_RETRIES_ID_ALLOC {
                let external_id: E = E::from(self.rng.next_u64());
                if !self.ids.contains_key(&external_id) {
                    self.ids.insert(external_id, internal_id);
                    return external_id;
                }
            }
            panic!("Could not find a valid task id");
        } else {
            // Use a Fibonacci sequence.
            let id: u64 = self.current_id;
            // Roll around.
            self.current_id = if self.current_id < u64::MAX - self.last_id {
                self.current_id + self.last_id
            } else {
                self.last_id - (u64::MAX - self.current_id)
            };
            self.last_id = id;
            let external_id: E = E::from(id);
            if self.ids.insert(external_id, internal_id).is_some() {
                panic!("Should not have a previous task with this id");
            }
            external_id
        }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.ids.len()
    }
}

#[cfg(feature = "direct-mapping")]
impl<E: Eq + Hash + From<u64> + Into<u64> + Copy, I: From<u64> + Into<u64> + Copy> IdMap<E, I> {
    #[inline(always)]
    pub fn get(&self, external_id: &E) -> Option<I> {
        Some(<E as Into<u64>>::into(*external_id).into())
    }

    #[inline(always)]
    pub fn remove(&mut self, external_id: &E) -> Option<I> {
        Some(<E as Into<u64>>::into(*external_id).into())
    }

    #[inline(always)]
    pub fn insert_with_new_id(&mut self, internal_id: I) -> E {
        E::from(internal_id.into())
    }
}

//======================================================================================================================
// Trait Implementations
//======================================================================================================================

#[cfg(not(feature = "direct-mapping"))]
impl<E: Eq + Hash + From<u64> + Into<u64> + Copy, I: From<u64> + Into<u64> + Copy> Default for IdMap<E, I> {
    fn default() -> Self {
        Self {
            // Don't need to pre-allocate, the overhead is a 6ns on the scheduler insert benchmark.
            ids: HashMap::<E, I>::with_capacity(DEFAULT_SIZE),
            rng: SmallRng::seed_from_u64(SCHEDULER_SEED),
            last_id: 1,
            current_id: 2,
        }
    }
}

#[cfg(feature = "direct-mapping")]
impl<E: Eq + Hash + From<u64> + Into<u64> + Copy, I: From<u64> + Into<u64> + Copy> Default for IdMap<E, I> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}
