#![allow(clippy::new_without_default)]

use core::future::Future;
use std::pin::Pin;

pub struct Coroutine {
    pub future: Pin<Box<dyn Future<Output = ()>>>
}

pub struct Koryto {
    pub coroutines: Vec<Coroutine>,
}

impl Koryto {
    pub fn new() -> Self {
        Self { coroutines: Vec::new() }
    }

    pub fn start(&mut self, co: impl Future<Output = ()> + 'static) {
        self.coroutines.push(Coroutine { future: Box::pin(co) });
    }

    pub fn poll_coroutines(&mut self, delta: f32) {
        for co in self.coroutines.iter_mut() {
            todo!();
        }
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
