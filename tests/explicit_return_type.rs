#![feature(use_extern_macros)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

#[derive(Debug)]
struct Cat;

#[derive(Debug)]
struct Dog;

trait Animal: ::std::fmt::Debug {}

impl Animal for Cat {}

impl Animal for Dog {}

#[::trace2::trace2]
fn new_animal(kind: &'static str) -> Box<Animal> {
    match kind {
        "cat" => Box::new(Cat),
        "dog" => Box::new(Dog),
        _ => panic!(),
    }
}

#[test]
fn test_explicit_return_type() {
    env_logger::Builder::from_default_env()
        .default_format_module_path(false)
        .init();

    new_animal("cat");
    new_animal("dog");
}
