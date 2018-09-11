# trace2

A procedural macro for tracing the execution of functions in Rust language (nightly).

The functionality is similar to the [trace] crate, but it is thread safe and uses [log] so that you can have more
control. In addition, unlike [trace], You don't need to declare any static variable. Just adding attributes to what you
want to trace and *it just works™*.

## Usage

1. Add dependency to your `Cargo.toml`:

   ```toml
   trace2 = "0.1"
   ```

2. Import crate:

   ```rust
   #![feature(use_extern_macros)]   // Not needed if your rustc is recent enough.
   extern crate trace2;
   ```

3. Add `#[::trace2::trace2]` attribute to the function, impl block or mod block.

   > Adding trace to a mod requires a recent rust nightly compiler with `proc_macro_mod` feature enabled.

## Examples

### Trace specific function

[examples/basic.rs](./examples/basic.rs): Trace specified function by adding trace2 attribute to the function.

```rust
#![feature(use_extern_macros)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

#[::trace2::trace2]
fn foo(a: i32, b: i32) {
    println!("I'm in foo!");
    bar((a, b));
}

#[::trace2::trace2]
fn bar((a, b): (i32, i32)) -> i32 {
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

### Trace whole impl block

[examples/impl_level.rs](./examples/impl_level.rs): Trace all functions in the impl block by adding trace2 attribute
to the impl block.

```rust
#![feature(use_extern_macros)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

struct Foo;

#[trace2::trace2]
impl Foo {
    fn foo(b: i32) -> i32 {
        b
    }

    fn bar(&self, a: i32) -> i32 {
        a
    }

    fn boz(&self, a: i32) -> i32 {
        self.bar(a)
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .default_format_module_path(false)
        .init();

    let foo = Foo;
    Foo::foo(2);
    foo.bar(7);
    foo.boz(13);
}
```

Output:

```
TRACE 2018-09-11T07:05:59Z: >>>> impl_level::Foo::foo(b: 2)
TRACE 2018-09-11T07:05:59Z: <<<< impl_level::Foo::foo = 2
TRACE 2018-09-11T07:05:59Z: >>>> impl_level::Foo::bar(a: 7)
TRACE 2018-09-11T07:05:59Z: <<<< impl_level::Foo::bar = 7
TRACE 2018-09-11T07:05:59Z: >>>> impl_level::Foo::boz(a: 13)
TRACE 2018-09-11T07:05:59Z: >>>>>>>> impl_level::Foo::bar(a: 13)
TRACE 2018-09-11T07:05:59Z: <<<<<<<< impl_level::Foo::bar = 13
TRACE 2018-09-11T07:05:59Z: <<<< impl_level::Foo::boz = 13
```

See more examples in the [examples](./examples) directory.

## TODO

- Support outputting impl type for nested trace attributes
- Support pausing

## License

MIT

[trace]: https://docs.rs/trace/
[log]: https://docs.rs/log/
