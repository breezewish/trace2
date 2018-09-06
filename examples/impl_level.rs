#![feature(use_extern_macros)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

use trace2::trace2;

struct Foo;

#[trace2]
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
