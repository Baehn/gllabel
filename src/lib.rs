use std::slice;

use bezier::{Bezier2, Vec2};

mod test_data;
mod bezier;
mod buffer;
pub mod grid;

// src/lib.rs
// #![feature(vec_into_raw_parts)]


struct State {}

impl State {
    pub fn set_curves(curves: &Vec<Bezier2>) {}
}


#[no_mangle]
pub extern "C" fn hello() {
    println!("Hello world!");
}

#[no_mangle]
pub extern "C" fn print(a: i32, b: f32, c: bool) {
    println!("int: {}, float: {}, bool: {}", a, b, c);
}

#[no_mangle]
pub extern "C" fn reverse(array: *mut u32, size: u32) -> *mut u32 {
    let mut vec = unsafe {
        assert!(!array.is_null());
        Vec::from_raw_parts(array, size as usize, size as usize)
    };

    vec.reverse();
    let (ptr, _) = ffi_utils::vec_into_raw_parts(vec);
    ptr
}

#[no_mangle]
pub extern "C" fn create() -> *mut Vec2 {
    let mut vec = Vec::new();

    vec.push(Vec2 { x: 1.0, y: 1.0 });
    vec.push(Vec2 { x: 2.0, y: 3.0 });

    let (ptr, _) = ffi_utils::vec_into_raw_parts(vec);
    ptr
}

