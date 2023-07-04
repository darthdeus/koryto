#![allow(clippy::new_without_default)]

use core::future::Future;
use std::{pin::Pin, task::{RawWaker, RawWakerVTable, Waker}};

pub struct Coroutine {
    pub future: Pin<Box<dyn Future<Output = ()> + 'static>>
}

pub struct Koryto {
    pub coroutines: Vec<Coroutine>,
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

impl Koryto {
    pub fn new() -> Self {
        Self { coroutines: Vec::new() }
    }

    pub fn start(&mut self, co: impl Future<Output = ()> + 'static) {
        self.coroutines.push(Coroutine { future: Box::pin(co) });
    }

    pub fn poll_coroutines(&mut self, _delta: f32) {
        let raw_waker = make_waker_vtable();
        let waker = unsafe { Waker::from_raw(raw_waker) };
        let mut context = std::task::Context::from_waker(&waker);

        self.coroutines.retain_mut(|co| {
            !co.future.as_mut().poll(&mut context).is_ready()
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{rc::Rc, cell::RefCell};

    use super::*;

    #[test]
    fn it_works() {
        let mut ko = Koryto::new();

        let val = Rc::new(RefCell::new(3));
        let val_inner = val.clone();

        ko.start(async move {
            *val_inner.borrow_mut() += 2;
            println!("foo!");
        });

        assert_eq!(*val.borrow(), 5);
    }
}
