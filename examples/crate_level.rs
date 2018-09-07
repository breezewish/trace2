#![feature(proc_macro_mod)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

mod implementation {
    #[::trace2::trace2]

    pub fn foo(a: i32, b: i32) {
        println!("I'm in foo!");
        sub::bar((a, b));
    }

    mod sub {
        pub fn bar((a, b): (i32, i32)) -> i32 {
            println!("I'm in bar!");
            if a == 1 {
                2
            } else {
                b
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .default_format_module_path(false)
        .init();

    implementation::foo(1, 2);
}
