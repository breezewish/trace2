#![feature(use_extern_macros)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

#[::trace2::trace2]
fn foo(a: i32) -> i32 {
    loop {
        return a + 1;
    }
}

#[test]
fn test_explicit_return_type() {
    env_logger::Builder::from_default_env()
        .default_format_module_path(false)
        .init();

    foo(5);
}
