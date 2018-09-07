// This case currently fails when using quote_spanned!.

#![feature(use_extern_macros)]
#![feature(proc_macro_path_invoc)]

extern crate trace2;
#[macro_use]
extern crate log;
extern crate env_logger;

struct Func {
    sig: FuncKind,
}

impl Func {
    fn dummy(&self, foo: Vec<u8>) {
        // nothing
    }
}

enum FuncKind {
    FloorInt,
    CeilInt,
    RoundInt,
    FloorReal,
    CeilReal,
    RoundReal,
}

macro_rules! dispatch_call {
    (
        INT_CALLS {$($i_sig:ident => $i_func:ident $($i_arg:expr)*,)*}
        REAL_CALLS {$($r_sig:ident => $r_func:ident $($r_arg:expr)*,)*}
    ) => {
        #[::trace2::trace2]
        impl Func {
            pub fn eval_int(&self, foo: Vec<u8>) {
                match self.sig {
                    $(FuncKind::$i_sig => self.$i_func(foo, $($i_arg),*)),*,
                    _ => panic!(),
                }
            }

            pub fn eval_real(&self, foo: Vec<u8>) {
                match self.sig {
                    $(FuncKind::$r_sig => self.$r_func(foo, $($r_arg),*),)*
                    _ => panic!(),
                }
            }

            pub fn eval(&self, foo: Vec<u8>) {
                match self.sig {
                    $(FuncKind::$i_sig => {
                        self.$i_func(foo, $($r_arg)*)
                    })*
                    $(FuncKind::$r_sig => {
                        self.$r_func(foo, $($r_arg)*)
                    })*
                    _ => unimplemented!(),
                }
            }
        }
    };
}

dispatch_call! {
    INT_CALLS {
        FloorInt => dummy,
        CeilInt => dummy,
        RoundInt => dummy,
    }
    REAL_CALLS {
        FloorReal => dummy,
        CeilReal => dummy,
        RoundReal => dummy,
    }
}

#[test]
fn test_inside_macro() {
    env_logger::Builder::from_default_env()
        .default_format_module_path(false)
        .init();

    let func = Func {
        sig: FuncKind::FloorReal,
    };
    func.eval(vec![1, 2, 3]);
}
