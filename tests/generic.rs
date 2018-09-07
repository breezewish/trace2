#![feature(use_extern_macros)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

#[::trace2::trace2]
fn boz<T>(arg1: T, arg2: T) -> impl ::std::fmt::Display
where
    T: PartialEq + ::std::fmt::Debug,
{
    println!("I'm in boz!");
    println!("arg1 == arg2 is {}", arg1 == arg2);
    "something can display"
}

#[test]
fn test_generic() {
    env_logger::Builder::from_default_env()
        .default_format_timestamp(false)
        .default_format_module_path(false)
        .init();

    boz(1, 2);
}
