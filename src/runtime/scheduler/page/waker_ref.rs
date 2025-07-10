// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Imports
//======================================================================================================================

use crate::runtime::scheduler::page::{page::WakerPage, WakerPageRef, WAKER_PAGE_SIZE};
use ::std::{
    mem,
    ptr::NonNull,
    task::{RawWaker, RawWakerVTable},
};

//======================================================================================================================
// Structures
//======================================================================================================================

/// This reference is a representation for the status of a particular task
/// stored in a [WakerPage].
#[repr(transparent)]
pub struct WakerRef(NonNull<u8>);

//======================================================================================================================
// Associate Functions
//======================================================================================================================

impl WakerRef {
    pub fn new(raw_page_ref: NonNull<u8>) -> Self {
        Self(raw_page_ref)
    }

    /// Casts the target [WakerRef] back into reference to a [WakerPage] plus an
    /// offset indicating the target task in the latter structure.
    ///
    /// For more information on this hack see comments on [crate::page::WakerPageRef].
    fn base_ptr(&self) -> (NonNull<WakerPage>, usize) {
        let ptr = self.0.as_ptr();
        let forward_offset = ptr.align_offset(WAKER_PAGE_SIZE);
        let mut base_ptr = ptr;
        let mut offset = 0;
        if forward_offset != 0 {
            offset = WAKER_PAGE_SIZE - forward_offset;
            base_ptr = ptr.wrapping_sub(offset);
        }
        unsafe { (NonNull::new_unchecked(base_ptr).cast(), offset) }
    }

    /// Sets the notification flag for the task that associated with the target [WakerRef].
    fn wake_by_ref(&self) {
        let (base_ptr, ix) = self.base_ptr();
        let base = unsafe { &*base_ptr.as_ptr() };
        base.notify(ix);
    }

    /// Sets the notification flag for the task that is associated with the target [WakerRef].
    fn wake(self) {
        self.wake_by_ref()
    }

    /// Gets the reference count of the target [WakerRef].
    #[cfg(test)]
    pub fn refcount(&self) -> u64 {
        let (base_ptr, _) = self.base_ptr();
        unsafe { base_ptr.as_ref().refcount() }
    }
}

//======================================================================================================================
// Trait Implementations
//======================================================================================================================

impl Clone for WakerRef {
    fn clone(&self) -> Self {
        let (base_ptr, _) = self.base_ptr();
        let page_ref = WakerPageRef::new(base_ptr);
        // Increment reference count.
        mem::forget(page_ref.clone());
        // This is not a double increment.
        mem::forget(page_ref);
        WakerRef(self.0)
    }
}

impl Drop for WakerRef {
    fn drop(&mut self) {
        let (base_ptr, _) = self.base_ptr();
        // Decrement the refcount.
        drop(WakerPageRef::new(base_ptr));
    }
}

impl Into<RawWaker> for WakerRef {
    fn into(self) -> RawWaker {
        let ptr = self.0.cast().as_ptr() as *const ();
        let waker = RawWaker::new(ptr, &VTABLE);
        // Increment reference count.
        mem::forget(self);
        waker
    }
}

unsafe fn waker_ref_clone(ptr: *const ()) -> RawWaker {
    let waker_ref = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    let out = waker_ref.clone();
    // Increment reference count.
    mem::forget(waker_ref);
    out.into()
}

unsafe fn waker_ref_wake(ptr: *const ()) {
    WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8)).wake();
}

unsafe fn waker_ref_wake_by_ref(ptr: *const ()) {
    let waker_ref = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    waker_ref.wake_by_ref();
    // Increment reference count.
    mem::forget(waker_ref);
}

unsafe fn waker_ref_drop(ptr: *const ()) {
    // Decrement reference count.
    drop(WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8)));
}

/// Raw Waker Trait Implementation for Waker References
pub const VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_ref_clone, waker_ref_wake, waker_ref_wake_by_ref, waker_ref_drop);

//======================================================================================================================
// Unit Tests
//======================================================================================================================

#[cfg(test)]
mod tests {
    use crate::runtime::scheduler::{
        page::{WakerPageRef, WakerRef},
        waker64::WAKER_BIT_LENGTH,
    };
    use ::anyhow::Result;
    use ::rand::Rng;
    use ::test::{black_box, Bencher};

    #[test]
    fn test_refcount() -> Result<()> {
        let page = WakerPageRef::default();
        crate::ensure_eq!(page.refcount(), 1);

        let page_clone = page.as_raw_waker_ref(0);
        crate::ensure_eq!(page.refcount(), 2);

        let waker_ref1 = WakerRef::new(page_clone);
        crate::ensure_eq!(waker_ref1.refcount(), 2);
        crate::ensure_eq!(page.refcount(), 2);

        let waker_ref2 = WakerRef::new(page.as_raw_waker_ref(31));
        crate::ensure_eq!(waker_ref2.refcount(), 3);
        crate::ensure_eq!(page.refcount(), 3);

        let waker_ref3 = waker_ref2.clone();
        crate::ensure_eq!(waker_ref3.refcount(), 4);
        crate::ensure_eq!(page.refcount(), 4);

        drop(waker_ref3);
        crate::ensure_eq!(page.refcount(), 3);

        drop(waker_ref2);
        crate::ensure_eq!(page.refcount(), 2);

        drop(waker_ref1);
        crate::ensure_eq!(page.refcount(), 1);

        Ok(())
    }

    #[test]
    fn test_wake() -> Result<()> {
        let page = WakerPageRef::default();
        crate::ensure_eq!(page.refcount(), 1);

        let waker_ref1 = WakerRef::new(page.as_raw_waker_ref(0));
        crate::ensure_eq!(page.refcount(), 2);

        let waker_ref2 = WakerRef::new(page.as_raw_waker_ref(31));
        crate::ensure_eq!(page.refcount(), 3);

        let waker_ref3 = WakerRef::new(page.as_raw_waker_ref(15));
        crate::ensure_eq!(page.refcount(), 4);

        waker_ref1.wake();
        crate::ensure_eq!(page.take_notified(), 1 << 0);
        crate::ensure_eq!(page.refcount(), 3);

        waker_ref2.wake();
        waker_ref3.wake();
        crate::ensure_eq!(page.take_notified(), 1 << 15 | 1 << 31);
        crate::ensure_eq!(page.refcount(), 1);

        Ok(())
    }

    #[test]
    fn test_wake_by_ref() -> Result<()> {
        let page = WakerPageRef::default();
        crate::ensure_eq!(page.refcount(), 1);

        let waker_ref1 = WakerRef::new(page.as_raw_waker_ref(0));
        crate::ensure_eq!(page.refcount(), 2);

        let waker_ref2 = WakerRef::new(page.as_raw_waker_ref(31));
        crate::ensure_eq!(page.refcount(), 3);

        let waker_ref3 = WakerRef::new(page.as_raw_waker_ref(15));
        crate::ensure_eq!(page.refcount(), 4);

        waker_ref1.wake_by_ref();
        crate::ensure_eq!(page.take_notified(), 1 << 0);
        crate::ensure_eq!(page.refcount(), 4);

        waker_ref2.wake_by_ref();
        waker_ref3.wake_by_ref();
        crate::ensure_eq!(page.take_notified(), 1 << 15 | 1 << 31);
        crate::ensure_eq!(page.refcount(), 4);

        drop(waker_ref3);
        crate::ensure_eq!(page.refcount(), 3);

        drop(waker_ref2);
        crate::ensure_eq!(page.refcount(), 2);

        drop(waker_ref1);
        crate::ensure_eq!(page.refcount(), 1);

        Ok(())
    }

    #[bench]
    fn wake_bench(b: &mut Bencher) {
        let page = WakerPageRef::default();
        let ix = rand::thread_rng().gen_range(0..WAKER_BIT_LENGTH);

        b.iter(|| {
            let raw_page_ref = black_box(page.as_raw_waker_ref(ix));
            let waker_ref = WakerRef::new(raw_page_ref);
            waker_ref.wake();
        });
    }

    #[bench]
    fn wake_by_ref_bench(b: &mut Bencher) {
        let page = WakerPageRef::default();
        let ix = rand::thread_rng().gen_range(0..WAKER_BIT_LENGTH);

        b.iter(|| {
            let raw_page_ref = black_box(page.as_raw_waker_ref(ix));
            let waker_ref = WakerRef::new(raw_page_ref);
            waker_ref.wake_by_ref();
        });
    }
}
