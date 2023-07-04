#![allow(clippy::new_without_default)]

use core::future::Future;
use std::{pin::Pin, sync::Arc};

use futures::task::{ArcWake, waker_ref};

pub struct Coroutine {
    pub future: Pin<Arc<dyn Future<Output = ()> + 'static + Send + Sync>>
}

impl ArcWake for Coroutine {
    fn wake_by_ref(_arc_self: &std::sync::Arc<Self>) {
        // TODO: nothing for now
    }
}

pub struct Koryto {
    pub coroutines: Vec<Coroutine>,
}

impl Koryto {
    pub fn new() -> Self {
        Self { coroutines: Vec::new() }
    }

    pub fn start(&mut self, co: impl Future<Output = ()> + 'static + Send + Sync) {
        self.coroutines.push(Coroutine { future: Arc::pin(co) });
    }

    pub fn poll_coroutines(&mut self, delta: f32) {
        self.coroutines.retain_mut(|co| {
            // :derppat:
            let f = co.future.as_mut();
            let waker = waker_ref(co.future);
            let context = std::task::Context::from_waker(&waker);

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
