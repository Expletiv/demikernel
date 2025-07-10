// Copyright(c) Microsoft Corporation.
// Licensed under the MIT license.

//! This module provides a small performance profiler for the Demikernel libOSes. It supports two kinds of profilers:
//! ones for synchronous blocks of code and ones for asynchronous blocks of code. For synchronous blocks, we use a
//! [SyncScopeGuard] to automatically scope the timer. The profiler creates the [SyncScopeGuard] when the timer starts
//! and the timer stops when the [SyncScopeGuard] goes out of scope. For asynchronous blocks, we simply create a
//! [SharedScope] and time the number of cycles for each poll. We do not use a guard for scoping asynchronous blocks.

//======================================================================================================================
// Exports
//======================================================================================================================

mod scope;
pub use crate::perftools::profiler::scope::AsyncScope;
#[cfg(test)]
mod tests;

//======================================================================================================================
// Imports
//======================================================================================================================

use crate::{
    perftools::profiler::scope::{SharedScope, SyncScopeGuard},
    runtime::{types::demi_callback_t, SharedObject},
};
use ::futures::future::FusedFuture;
use ::std::{
    io,
    ops::{Deref, DerefMut},
    pin::Pin,
    thread,
    time::SystemTime,
};

//======================================================================================================================
// Structures
//======================================================================================================================

thread_local!(
    pub static PROFILER: SharedProfiler = SharedProfiler::default();
);

/// A `Profiler` stores the scope tree and keeps track of the currently active scope. Note that there is a global
/// thread-local instance of `Profiler` in [`PROFILER`](constant.PROFILER.html), so it is not possible to manually
/// create an instance of `Profiler`.
pub struct Profiler {
    roots: Vec<SharedScope>,
    current_sync: Option<SharedScope>,
    current_async: Option<SharedScope>,
    callback: Option<demi_callback_t>,
}

#[derive(Clone)]
pub struct SharedProfiler(SharedObject<Profiler>);

//======================================================================================================================
// Associated Functions
//======================================================================================================================

pub fn reset() {
    PROFILER.with(|p| p.clone().reset());
}

pub fn set_callback(callback: demi_callback_t) {
    PROFILER.with(|p| p.clone().set_callback(callback));
}

/// Create a special async scopes that is rooted because it does not run under other scopes.
#[inline]
pub async fn coroutine_scope<F: FusedFuture>(name: &'static str, mut coroutine: Pin<Box<F>>) -> F::Output {
    AsyncScope::new(name, coroutine.as_mut()).await
}

impl Profiler {
    fn write<W: io::Write>(&self, out: &mut W) -> io::Result<()> {
        let thread_id = thread::current().id();
        let ns_per_cycle = Self::measure_ns_per_cycle();

        const HEADER_ROW: &'static str = "call_depth,thread_id,function_name,num_calls,cycles_per_call,nanoseconds_per_call,total_duration,total_duration_exclusive";
        writeln!(out, "{}", HEADER_ROW)?;

        for scope in self.roots.iter() {
            scope.write_recursive(out, thread_id, 0, ns_per_cycle)?;
        }

        out.flush()
    }

    fn measure_ns_per_cycle() -> f64 {
        let start_ts = SystemTime::now();
        let start_cycles = unsafe { x86::time::rdtscp().0 };

        // dummy calculations for measurement
        test::black_box((0..10000).fold(0, |old, new| old ^ new));

        let end_cycles = unsafe { x86::time::rdtscp().0 };
        let duration = SystemTime::now().duration_since(start_ts).expect("Time went backwards");
        let duration_in_ns = duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64;
        let ns_per_cycle = duration_in_ns as f64 / (end_cycles - start_cycles) as f64;

        ns_per_cycle
    }
}

impl SharedProfiler {
    pub fn set_callback(&mut self, callback: demi_callback_t) {
        self.callback = Some(callback)
    }

    fn find_or_create_new_scope(
        scopes: &mut Vec<SharedScope>,
        name: &'static str,
        parent: Option<SharedScope>,
        callback: Option<demi_callback_t>,
    ) -> SharedScope {
        match scopes.iter().find(|s| s.name == name) {
            Some(s) => s.clone(),
            None => {
                let new_scope = SharedScope::new(name, parent, callback);
                scopes.push(new_scope.clone());
                new_scope
            },
        }
    }

    /// Create and enter a syncronous scope. Returns a [`Guard`](struct.Guard.html) that should be dropped upon
    /// leaving the scope. Usually, this method will be called by the [`profile`](macro.profile.html) macro,
    /// so it does not need to be used directly.
    fn create_scope(&mut self, current: &mut Option<SharedScope>, name: &'static str) -> SharedScope {
        let callback = self.callback;
        match current {
            Some(current) => {
                let parent = Some(current.clone());
                Self::find_or_create_new_scope(&mut current.children, name, parent, callback)
            },
            None => Self::find_or_create_new_scope(&mut self.roots, name, None, callback),
        }
    }

    pub fn enter_sync(&mut self, name: &'static str) -> SyncScopeGuard {
        let mut current = self.current_sync.clone();
        let scope = self.create_scope(&mut current, name);
        self.current_sync = Some(scope.clone());
        scope.enter_sync()
    }

    #[inline]
    fn leave_sync(&mut self, duration: u64) {
        // Note that we could now still be anywhere in the previous profiling
        // tree, so we can not simply reset `self.current`. However, as the
        // frame comes to an end we will eventually leave a root node, at which
        // point `self.current` will be set to `None`.
        self.current_sync = if let Some(mut current) = self.current_sync.take() {
            current.add_duration(duration);
            let parent = current.parent.as_ref().cloned();
            parent
        } else {
            // This should not happen with proper usage.
            unreachable!("Called perftools::profiler::leave_sync() while not in any scope");
        };
    }

    pub fn enter_async(&mut self, name: &'static str) {
        let mut current = self.current_async.clone();
        let scope = self.create_scope(&mut current, name);
        self.current_async = Some(scope.clone());
    }

    #[inline]
    fn leave_async(&mut self, duration: u64) {
        self.current_async = if let Some(mut current) = self.current_async.take() {
            current.add_duration(duration);
            let parent = current.parent.as_ref().cloned();
            parent
        } else {
            // This should not happen with proper usage.
            unreachable!("Called perftools::profiler::leave_async() while not in any scope");
        };
    }

    fn reset(&mut self) {
        self.roots.clear();
    }
}

//======================================================================================================================
// Trait Implementations
//======================================================================================================================

impl Deref for SharedProfiler {
    type Target = Profiler;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SharedProfiler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for SharedProfiler {
    fn default() -> Self {
        Self(SharedObject::new(Profiler {
            roots: Vec::new(),
            current_sync: None,
            current_async: None,
            callback: None,
        }))
    }
}

impl Drop for Profiler {
    fn drop(&mut self) {
        self.write(&mut std::io::stdout()).expect("failed to write to stdout");
    }
}
