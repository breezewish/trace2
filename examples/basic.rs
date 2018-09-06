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

#[trace2]
pub fn boz<T>(arg1: T, arg2: T) -> impl ::std::fmt::Display
where
    T: PartialEq + ::std::fmt::Debug
{
    println!("I'm in boz!");
    println!("arg1 == arg2 is {}", arg1 == arg2);
    "something can display"
}

fn main() {
    env_logger::init();
    foo(1, 2);
    boz(1, 2);
}
