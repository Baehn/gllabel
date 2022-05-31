// src/lib.rs
#[no_mangle]
pub extern fn hello() {
    println!("Hello world!");
}

#[no_mangle]
pub extern fn print(a: i32, b: f32, c: bool) {
    println!("int: {}, float: {}, bool: {}", a, b, c);
}

