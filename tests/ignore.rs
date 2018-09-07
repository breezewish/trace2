#![feature(use_extern_macros)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

struct Foo;

struct Bar(i32);

#[::trace2::trace2]
impl Foo {
    fn fun1(&self) -> i32 {
        1
    }

    fn fun2(&self) -> i32 {
        2
    }

    #[::trace2::trace2(ignore)]
    fn fun3(self) -> Bar {
        Bar(3)
    }
}

#[test]
fn test_ignore() {
    env_logger::Builder::from_default_env()
        .default_format_timestamp(false)
        .default_format_module_path(false)
        .init();

    let foo = Foo;
    assert_eq!(foo.fun1(), 1);
    assert_eq!(foo.fun2(), 2);
    assert_eq!(foo.fun3().0, 3);
}
