# trace2

A procedural macro for tracing the execution of functions in Rust language (nightly).

The functionality is similar to the [trace] crate, but it is thread safe and uses [log] so that you can have more
control. In addition, unlike [trace], You don't need to declare any static variable. Just adding attributes to what you
want to trace and *it just works™*.

## Example

You can add `#[trace2]` before a function or impl block, or add `#![trace2]` in a mod.

Adding `#![trace2]` in a mod requires a recent rust nightly compiler with `proc_macro_mod` feature enabled.

```rust
#![feature(use_extern_macros)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

use trace2::trace2;

#[trace2]
pub fn foo(a: i32, b: i32) {
    println!("I'm in foo!");
    bar((a, b));
}

#[trace2]
pub fn bar((a, b): (i32, i32)) -> i32 {
    println!("I'm in bar!");
    if a == 1 {
        2
    } else {
        b
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .default_format_module_path(false)
        .init();

    foo(1, 2);
}
```

Output:

```
TRACE 2018-09-06T17:06:54Z: >>>> basic::foo(a: 1, b: 2)
I'm in foo!
TRACE 2018-09-06T17:06:54Z: >>>>>>>> basic::bar(a: 1, b: 2)
I'm in bar!
TRACE 2018-09-06T17:06:54Z: <<<<<<<< basic::bar = 2
TRACE 2018-09-06T17:06:54Z: <<<< basic::foo = ()
```

## TODO

- Support pausing

## License

MIT

[trace]: https://docs.rs/trace/
[log]: https://docs.rs/log/