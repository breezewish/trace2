#![feature(use_extern_macros)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

#[derive(Clone)]
struct Foo;

struct Bar(i32);

#[::trace2::trace2]
impl Foo {
    fn fun1(&self) -> i32 {
        println!("fun2 = {}", self.fun2());
        1
    }

    fn fun2(&self) -> i32 {
        println!("fun3 = {}", self.clone().fun3().0);
        2
    }

    #[::trace2::trace2(ignore)]
    fn fun3(self) -> Bar {
        Bar(3)
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .default_format_module_path(false)
        .init();

    let foo = Foo;
    foo.fun1();
}
