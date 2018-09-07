#![feature(proc_macro_mod)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

mod implementation {
    #![::trace2::trace2]

    #[::trace2::trace2]
    pub fn foo(a: i32, b: i32) {
        sub::bar((a, b));
        let cat = Cat;
        assert_eq!(cat.nyan(), 7);
    }

    #[derive(Debug)]
    struct Cat;

    #[::trace2::trace2(ignore)]
    impl Cat {
        #[::trace2::trace2]
        fn nyan(&self) -> i32 {
            let dog = Dog;
            assert_eq!(dog.bite(), 12);
            7
        }
    }

    #[derive(Debug)]
    struct Dog;

    #[::trace2::trace2]
    impl Dog {
        fn bite(&self) -> i32 {
            12
        }
    }

    mod sub {
        #[::trace2::trace2]
        pub fn bar((a, b): (i32, i32)) -> i32 {
            if a == 1 {
                2
            } else {
                b
            }
        }
    }
}

#[test]
fn test_nested() {
    env_logger::Builder::from_default_env()
        .default_format_timestamp(false)
        .default_format_module_path(false)
        .init();

    implementation::foo(1, 2);
}
