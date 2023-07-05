use koryto::*;
use std::{cell::RefCell, rc::Rc};

#[test]
fn no_send_required() {
    let mut ko = Koryto::new();

    // Koryto doesn't require Send/Sync, so coroutines can
    // work with raw pointers without a mutex.
    let p: *const i32 = std::ptr::null();

    ko.start(async move {
        println!("happy pointer 🐷 = {:?}", p);
    });
}

#[test]
fn yield_frame_test() {
    let mut ko = Koryto::new();

    let val = Rc::new(RefCell::new(3));
    let val_inner = val.clone();

    assert_eq!(*val.borrow(), 3);

    ko.start(async move {
        *val_inner.borrow_mut() = 5;

        yield_frame().await;

        *val_inner.borrow_mut() = 7;
    });

    assert_eq!(*val.borrow(), 3);

    ko.poll_coroutines(0.0);
    assert_eq!(*val.borrow(), 5);

    ko.poll_coroutines(0.0);
    assert_eq!(*val.borrow(), 7);

    ko.poll_coroutines(0.0);
    assert_eq!(*val.borrow(), 7);
}

#[test]
fn yield_frame_immediately_poll_test() {
    let mut ko = Koryto::new();

    let val = Rc::new(RefCell::new(3));
    let val_inner = val.clone();

    assert_eq!(*val.borrow(), 3);

    ko.start_and_poll(async move {
        *val_inner.borrow_mut() = 5;

        yield_frame().await;

        *val_inner.borrow_mut() = 7;
    });

    assert_eq!(*val.borrow(), 5);

    ko.poll_coroutines(0.0);
    assert_eq!(*val.borrow(), 7);

    ko.poll_coroutines(0.0);
    assert_eq!(*val.borrow(), 7);
}

#[test]
fn wait_seconds_test() {
    let mut ko = Koryto::new();

    let val = Rc::new(RefCell::new(3));
    let val_inner = val.clone();

    ko.start_and_poll(async move {
        wait_seconds(0.5).await;

        *val_inner.borrow_mut() = 9
    });

    // Delta time obtained from the game loop
    let dt = 0.2;

    ko.poll_coroutines(dt);
    assert_eq!(*val.borrow(), 3);

    ko.poll_coroutines(dt);
    assert_eq!(*val.borrow(), 3);

    ko.poll_coroutines(dt);
    assert_eq!(*val.borrow(), 9);

    ko.poll_coroutines(dt);
    assert_eq!(*val.borrow(), 9);
}

#[test]
fn stop_coroutine_test() {
    let mut ko = Koryto::new();

    let val = Rc::new(RefCell::new(3));
    let val_inner = val.clone();

    assert_eq!(*val.borrow(), 3);

    let coroutine = ko.start(async move {
        *val_inner.borrow_mut() = 5;

        yield_frame().await;

        *val_inner.borrow_mut() = 7;
    });

    assert_eq!(*val.borrow(), 3);

    ko.poll_coroutines(0.0);
    assert_eq!(*val.borrow(), 5);

    ko.stop(coroutine);

    ko.poll_coroutines(0.0);
    assert_eq!(*val.borrow(), 5);

    ko.poll_coroutines(0.0);
    assert_eq!(*val.borrow(), 5);
}

#[test]
fn coroutine_cleanup_test() {
    let mut ko = Koryto::new();

    ko.start(async move {
        yield_frame().await;
    });

    assert_eq!(ko.active_coroutines(), 1);

    ko.poll_coroutines(0.0);
    assert_eq!(ko.active_coroutines(), 1);

    ko.poll_coroutines(0.0);
    assert_eq!(ko.active_coroutines(), 0);
}

#[test]
fn coroutine_cleanup_early_return_test() {
    let mut ko = Koryto::new();

    let val = Rc::new(RefCell::new(false));
    let val_inner = val.clone();

    ko.start(async move {
        if *val_inner.borrow() {
            return;
        }

        yield_frame().await;
    });

    *val.borrow_mut() = true;
    assert_eq!(ko.active_coroutines(), 1);

    ko.poll_coroutines(0.0);
    assert_eq!(ko.active_coroutines(), 0);

    ko.poll_coroutines(0.0);
    assert_eq!(ko.active_coroutines(), 0);
}

#[test]
fn select_test() {
    let mut ko = Koryto::new();

    let val = Rc::new(RefCell::new(0));
    let val_inner = val.clone();

    ko.start(async move {
        select! {
            _ = wait_seconds(0.5) => {
                *val_inner.borrow_mut() = 1;
            }
            _ = yield_frame() => {
                *val_inner.borrow_mut() = 2;
            }
        }
    });

    assert_eq!(*val.borrow(), 0);

    ko.poll_coroutines(0.2);
    assert_eq!(*val.borrow(), 0);

    ko.poll_coroutines(0.2);
    assert_eq!(*val.borrow(), 2);

    ko.poll_coroutines(0.2);
    assert_eq!(*val.borrow(), 2);
}
