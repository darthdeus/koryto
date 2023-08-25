# 🐷 Koryto 🐷

[![Crates.io](https://img.shields.io/crates/v/koryto.svg)](https://crates.io/crates/koryto)
[![Documentation](https://docs.rs/koryto/badge.svg)](https://docs.rs/koryto)
[![Build Status](https://github.com/darthdeus/koryto/actions/workflows/build.yaml/badge.svg)](https://github.com/darthdeus/koryto/actions)
[![License](https://img.shields.io/crates/l/koryto.svg)](https://github.com/darthdeus/koryto/blob/main/LICENSE)

> Pronounced like _corrito_, which is pronounced as if you combined _coroutine_ and _burrito_, because everyone knows coroutines are [burritos](https://blog.plover.com/prog/burritos.html) in the category of endofunctors.

Game loop focused async executor for all your coroutine needs. Inspired by [macroquad's experimental coroutines](https://docs.rs/macroquad/latest/macroquad/experimental/coroutines/), [the cosync crate](https://docs.rs/cosync/latest/cosync/), [Unity's amazing coroutines](https://docs.unity3d.com/ScriptReference/MonoBehaviour.StartCoroutine.html), and lastly [Godot's coroutines](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_basics.html#awaiting-for-signals-or-coroutines) which for a while [weren't true burritos](https://github.com/godotengine/godot/issues/24311).

![Pigs eating from a trough](./docs/img/logo.png)

> Koryto in Czech means trough, which is the thingy pigs eat from.

Koryto is single threaded and simple. The executor context `Koryto` expects to be polled every frame with the game's _delta time_.

## Example

```rust
// Create a new instance of the executor
let mut ko = Koryto::new();

// Shared state the coroutine sets
let val = Rc::new(RefCell::new(3));
let val_inner = val.clone();

// Start a new coroutine
ko.start(async move {
    wait_seconds(0.5).await;

    *val_inner.borrow_mut() = 9
});

// Delta time obtained from the game loop
let dt = 0.2;

// Poll coroutines as part of the game loop
ko.poll_coroutines(dt);
assert_eq!(*val.borrow(), 3);

ko.poll_coroutines(dt);
assert_eq!(*val.borrow(), 3);

// At this point 0.5 seconds have passed and we observe the coroutine be
// resumed and set the value.
ko.poll_coroutines(dt);
assert_eq!(*val.borrow(), 9);

ko.poll_coroutines(dt);
assert_eq!(*val.borrow(), 9);

```

## FAQ

### Is this stable?

No.

### Does it work?

Yes, probably ... [we have tests](https://github.com/darthdeus/koryto/blob/master/tests/basic_test.rs).

The source code should be simple enough for anyone to read and understand.
There's no 100 000 lines of code like in tokio. Currently `koryto` does
depend on `futures` in order to implement the `select!/join!` combinators.

### Why not just use `cosync` when this does exactly the same thing?

Unlike [`cosync`](https://docs.rs/cosync/latest/cosync/), `koryto` assumes your application is single thread,
which means your futures don't need to be `Send`.

This allows you to use `Rc` for shared state, and also just pass around pointers
into the futures without any wrappers.

`koryto` is also a bit simpler in its API and closer to [Macroquad's
coroutines](https://docs.rs/macroquad/latest/macroquad/experimental/coroutines/index.html),
although macroquad's futures also need to be `Send`.

## License

`koryto` is dual licensed:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
