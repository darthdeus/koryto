#![allow(clippy::new_without_default)]

use core::future::Future;

pub struct Koryto {}

impl Koryto {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&mut self, _co: impl Future<Output = ()>) {}
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
