// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! Implementation of our efficient, single-threaded task scheduler.
//!
//! Our scheduler uses a pinned memory slab to store tasks ([SchedulerFuture]s).
//! As background tasks are polled, they notify task in our scheduler via the
//! [crate::page::WakerPage]s.

//======================================================================================================================
// Imports
//======================================================================================================================

use crate::{
    collections::pin_slab::PinSlab,
    expect_some,
    runtime::scheduler::{
        page::{WakerPageRef, WakerRef},
        waker64::{WAKER_BIT_LENGTH, WAKER_BIT_LENGTH_SHIFT},
        ScheduleResult, Task,
    },
};
use ::bit_iter::BitIter;
use ::futures::Future;
use ::std::{
    iter::Flatten,
    pin::Pin,
    task::{Context, Waker},
    vec::IntoIter,
};

//======================================================================================================================
// Structures
//======================================================================================================================

/// Resource management group; all tasks belong to one.
/// By default, tasks inherit the allocator's group.
#[derive(Default)]
pub struct TaskGroup {
    /// All scheduled tasks.
    tasks: PinSlab<Box<dyn Task>>,
    /// Waker pages controlling scheduling.
    pages: Vec<WakerPageRef>,
    ready_tasks: Flatten<IntoIter<Vec<usize>>>,
}

//======================================================================================================================
// Associate Functions
//======================================================================================================================

impl TaskGroup {
    pub fn remove(&mut self, idx: usize) -> Option<Box<dyn Task>> {
        let (page_idx, offset) = self.locate(idx)?;
        self.pages[page_idx].clear(offset);

        if let Some(task) = self.tasks.remove_unpin(idx) {
            trace!(
                "remove(): name={:?}, id={:?}, pin_slab_index={:?}",
                task.name(),
                task.id(),
                idx
            );
            Some(task)
        } else {
            warn!("Unable to unpin and remove: pin_slab_index={:?}", idx);
            None
        }
    }

    pub fn schedule(&mut self, task: Box<dyn Task>) -> Option<ScheduleResult> {
        let name = task.name();
        let idx = self.tasks.insert(task)?;

        self.grow_pages(idx);

        let (page_idx, offset) = self.locate(idx)?;
        self.pages[page_idx].initialize(offset);

        trace!("schedule(): name={:?}, idx={:?}", name, idx);

        match self.run_task(idx) {
            Some(task) => Some(ScheduleResult::Completed(task)),
            None => Some(ScheduleResult::Scheduled(idx.into())),
        }
    }

    pub fn get_task_mut(&mut self, idx: usize) -> Option<Pin<&mut Box<dyn Task>>> {
        self.tasks.get_pin_mut(idx)
    }

    /// Computes the page and page offset of a given task based on its total offset.
    fn locate(&self, pin_slab_index: usize) -> Option<(usize, usize)> {
        // This check ensures that slab slot is actually occupied but trusts that pin_slab_index is for this task.
        if !self.tasks.contains(pin_slab_index) {
            return None;
        }
        let index = pin_slab_index >> WAKER_BIT_LENGTH_SHIFT;
        let offset = Self::get_page_offset(pin_slab_index);
        Some((index, offset))
    }

    /// Grow pages to fit the given pin slab index.
    fn grow_pages(&mut self, idx: usize) {
        while idx >= (self.pages.len() << WAKER_BIT_LENGTH_SHIFT) {
            self.pages.push(WakerPageRef::default());
        }
    }

    fn get_page_offset(pin_slab_index: usize) -> usize {
        pin_slab_index & (WAKER_BIT_LENGTH - 1)
    }

    fn get_pin_slab_index(page_idx: usize, page_offset: usize) -> usize {
        (page_idx << WAKER_BIT_LENGTH_SHIFT) + page_offset
    }

    fn pinned_task(&mut self, idx: usize) -> Pin<&mut Box<dyn Task>> {
        expect_some!(self.tasks.get_pin_mut(idx), "Invalid offset: {:?}", idx)
    }

    pub fn get_waker(&self, task_offset: usize) -> Option<Waker> {
        let (index, offset) = self.locate(task_offset)?;
        let raw_waker = self.pages[index].as_raw_waker_ref(offset);
        Some(unsafe { Waker::from_raw(WakerRef::new(raw_waker).into()) })
    }

    // Always check for new tasks on the first iteration; afterward, based on refill.
    pub fn run(&mut self, limit: Option<usize>, refill: bool) -> Vec<Box<dyn Task>> {
        let mut iter = 0;
        let mut tasks = Vec::new();

        while let Some(offset) = self.next(iter == 0 || refill) {
            if let Some(task) = self.run_task(offset) {
                tasks.push(task);
            }

            if limit.is_some_and(|max| iter >= max) {
                return tasks;
            }

            iter += 1
        }

        tasks
    }

    fn next(&mut self, refill: bool) -> Option<usize> {
        if let Some(task) = self.ready_tasks.next() {
            return Some(task);
        }

        if refill {
            self.ready_tasks = self.refill_ready();
            return self.ready_tasks.next();
        }

        None
    }

    /// Scan all waker pages for notified tasks.
    fn refill_ready(&mut self) -> Flatten<IntoIter<Vec<usize>>> {
        let mut out = vec![];

        for i in 0..self.pages.len() {
            let bits = self.pages[i].take_notified();
            out.push(BitIter::from(bits).map(|x| Self::get_pin_slab_index(i, x)).collect())
        }

        out.into_iter().flatten()
    }

    // Poll a ready task; return it if done.
    fn run_task(&mut self, idx: usize) -> Option<Box<dyn Task>> {
        let waker = self.get_waker(idx)?;
        let mut cx = Context::from_waker(&waker);
        let mut ptr = self.pinned_task(idx);
        let task = unsafe { Pin::new_unchecked(&mut *ptr) };

        if Future::poll(task, &mut cx).is_ready() {
            return self.remove(idx);
        }

        None
    }
}
