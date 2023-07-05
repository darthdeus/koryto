#![allow(clippy::new_without_default)]

use core::future::Future;
use std::{
    cell::Cell,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use thunderdome::{Arena, Index};

pub use futures;
pub use futures::{future::FusedFuture, join, select};

#[derive(Copy, Clone, Debug)]
pub struct Coroutine {
    id: Index,
}

struct CoroutineState {
    pub future: Pin<Box<dyn Future<Output = ()> + 'static>>,
}

thread_local! {
    static DELTA: Cell<f32> = Cell::new(0.0);
}

pub struct TimeDelayFuture {
    pub remaining: f32,
}

impl Future for TimeDelayFuture {
    type Output = Option<()>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        DELTA.with(|delta| {
            self.remaining -= delta.get();

            if self.remaining <= 0.0 {
                Poll::Ready(Some(()))
            } else {
                Poll::Pending
            }
        })
    }
}

impl FusedFuture for TimeDelayFuture {
    fn is_terminated(&self) -> bool {
        false
    }
}

pub fn wait_seconds(seconds: f32) -> TimeDelayFuture {
    TimeDelayFuture { remaining: seconds }
}

pub struct FrameFuture {
    pub is_done: bool,
}

impl Future for FrameFuture {
    type Output = Option<()>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.is_done {
            Poll::Ready(Some(()))
        } else {
            self.is_done = true;
            Poll::Pending
        }
    }
}

impl FusedFuture for FrameFuture {
    fn is_terminated(&self) -> bool {
        false
    }
}

pub fn yield_frame() -> FrameFuture {
    FrameFuture { is_done: false }
}

fn make_waker_vtable() -> RawWaker {
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &VTABLE)
    }
    unsafe fn wake(_data: *const ()) {
        panic!("wake is not supported");
    }
    unsafe fn wake_by_ref(_data: *const ()) {
        panic!("wake_by_ref is not supported");
    }
    unsafe fn drop(_data: *const ()) {}

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    RawWaker::new(std::ptr::null(), &VTABLE)
}

fn poll<T>(future: &mut Pin<Box<dyn Future<Output = T>>>) -> Option<T> {
    let raw_waker = make_waker_vtable();
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut context = Context::from_waker(&waker);

    match future.as_mut().poll(&mut context) {
        Poll::Ready(val) => Some(val),
        Poll::Pending => None,
    }
}

pub struct Koryto {
    coroutines: Arena<CoroutineState>,
}

impl Koryto {
    pub fn new() -> Self {
        Self {
            coroutines: Arena::new(),
        }
    }

    /// Spawns the coroutine but doesn't immediately poll. This is the default
    /// behavior other Rust async executors.
    pub fn start(&mut self, future: impl Future<Output = ()> + 'static) -> Coroutine {
        self.start_internal(future, false)
    }

    /// Spawns the coroutine and immediately polls it before returning.
    pub fn start_and_poll(&mut self, future: impl Future<Output = ()> + 'static) -> Coroutine {
        self.start_internal(future, true)
    }

    fn start_internal(
        &mut self,
        future: impl Future<Output = ()> + 'static,
        immediately_resume: bool,
    ) -> Coroutine {
        let mut state = CoroutineState {
            future: Box::pin(future),
        };

        if immediately_resume {
            poll(&mut state.future);
        }

        Coroutine {
            id: self.coroutines.insert(state),
        }
    }

    pub fn active_coroutines(&self) -> usize {
        self.coroutines.len()
    }

    pub fn stop(&mut self, coroutine: Coroutine) {
        self.coroutines.remove(coroutine.id);
    }

    pub fn poll_coroutines(&mut self, delta: f32) {
        DELTA.with(|delta_cell| {
            delta_cell.set(delta);
        });

        self.coroutines
            .retain(|_, co| poll(&mut co.future).is_none());
    }
}
