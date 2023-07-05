#![allow(clippy::new_without_default)]

use core::future::Future;
use std::{
    cell::RefCell,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

pub struct Coroutine {
    pub future: Pin<Box<dyn Future<Output = ()> + 'static>>,
}

pub struct Koryto {
    pub coroutines: Vec<Coroutine>,
}

thread_local! {
    static DELTA: RefCell<f32> = RefCell::new(0.0);
}

pub struct TimeDelayFuture {
    pub remaining: f32,
}

impl Future for TimeDelayFuture {
    type Output = Option<()>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        DELTA.with(|delta| {
            self.remaining -= *delta.borrow();

            if self.remaining <= 0.0 {
                Poll::Ready(Some(()))
            } else {
                Poll::Pending
            }
        })
    }
}

pub fn wait_seconds(seconds: f32) -> TimeDelayFuture {
    TimeDelayFuture { remaining: seconds }
}

pub struct FrameFuture {
    pub ready: bool,
}

impl Future for FrameFuture {
    type Output = Option<()>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.ready {
            Poll::Ready(Some(()))
        } else {
            self.ready = true;
            Poll::Pending
        }
    }
}

pub fn yield_frame() -> FrameFuture {
    FrameFuture { ready: false }
}

fn make_waker_vtable() -> RawWaker {
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &VTABLE)
    }
    unsafe fn wake(_data: *const ()) {}
    unsafe fn wake_by_ref(_data: *const ()) {}
    unsafe fn drop(_data: *const ()) {}

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    RawWaker::new(std::ptr::null(), &VTABLE)
}

fn resume<T>(future: &mut Pin<Box<dyn Future<Output = T>>>) -> Option<T> {
    let raw_waker = make_waker_vtable();
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut context = Context::from_waker(&waker);

    match future.as_mut().poll(&mut context) {
        Poll::Ready(val) => Some(val),
        Poll::Pending => None,
    }
}

impl Koryto {
    pub fn new() -> Self {
        Self {
            coroutines: Vec::new(),
        }
    }

    pub fn start(&mut self, co: impl Future<Output = ()> + 'static) {
        let mut co = Coroutine {
            future: Box::pin(co),
        };

        if resume(&mut co.future).is_none() {
            self.coroutines.push(co);
        }
    }

    pub fn poll_coroutines(&mut self, delta: f32) {
        DELTA.with(|delta_cell| {
            *delta_cell.borrow_mut() = delta;
        });

        let raw_waker = make_waker_vtable();
        let waker = unsafe { Waker::from_raw(raw_waker) };
        let mut context = Context::from_waker(&waker);

        self.coroutines
            .retain_mut(|co| !co.future.as_mut().poll(&mut context).is_ready());
    }
}
