#![feature(use_extern_macros)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

trait Foo {
    fn foo1(&self) -> i32;
}

// TODO: We need a prettier print for this kind
#[::trace2::trace2]
impl Foo for [&'static str; 2] {
    fn foo1(&self) -> i32 {
        self.len() as i32
    }
}

struct Bar(i32);

// TODO: Improve output for this kind
#[::trace2::trace2]
impl Foo for ::Bar {
    fn foo1(&self) -> i32 {
        self.0
    }
}

struct Boz(i32);

#[::trace2::trace2]
impl Foo for self::Boz {
    fn foo1(&self) -> i32 {
        self.0
    }
}

struct Alice<T>(T);

// TODO: We need a prettier print for this kind
#[::trace2::trace2]
impl Foo for Alice<i32> {
    fn foo1(&self) -> i32 {
        self.0
    }
}

#[test]
fn test_ignore() {
    env_logger::Builder::from_default_env()
        .default_format_timestamp(false)
        .default_format_module_path(false)
        .init();

    ["foo", "bar"].foo1();
    Bar(3).foo1();
    Boz(7).foo1();
    Alice(13).foo1();
}
