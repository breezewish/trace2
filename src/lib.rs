#![feature(use_extern_macros)]

extern crate trace2_macro;

pub use trace2_macro::trace2;

use std::cell::Cell;

thread_local! {
    pub static FUNC_CALL_LEVEL: Cell<usize> = Cell::new(0);
}
