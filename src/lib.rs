use std::slice;

// src/lib.rs
// #![feature(vec_into_raw_parts)]

#[repr(C)]
pub struct Vec2 {
    x: f32,
    y: f32,
}

#[repr(C)]
struct Bezier2 {
    e0: Vec2,
    e1: Vec2,
    c: Vec2,
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

const kBezierIndexUnused: u8 = 0;

#[no_mangle]
pub extern "C" fn r_write_vgrid_cell_to_buffer(
    // std::vector<std::vector<size_t>> &cellBeziers,
    // std::vector<std::vector<size_t>> &cellBeziers,
    // std::vector<char> &cellMids,
    // size_t cellIdx,
    data_ptr: *mut u8,
    depth: u8,
) {
    std::vector<size_t> *beziers = &cellBeziers[cellIdx];

    // Clear texel
    let data = unsafe { 
        assert!(!data_ptr.is_null());
        slice::from_raw_parts_mut(data_ptr, depth as usize) };
    for item in data.iter_mut() {
        *item = kBezierIndexUnused;
    }


    // Write out bezier indices to atlas texel
    let mut i: usize = 0;
    // size_t nbeziers = std::min(beziers->size(), (size_t)depth);
    // auto end = beziers->begin();
    // std::advance(end, nbeziers);
    // for (auto it = beziers->begin(); it != end; it++) {
    // 	// TODO: The uint8_t cast wont overflow because the bezier
    // 	// limit is checked when loading the glyph. But try to encode
    // 	// that info into the data types so no cast is needed.
    // 	data[i] = (uint8_t)(*it) + kBezierIndexFirstReal;
    // 	i++;
    // }

    // bool midInside = cellMids[cellIdx];

    // // Because the order of beziers doesn't matter and a single bezier is
    // // never referenced twice in one cell, metadata can be stored by
    // // adjusting the order of the bezier indices. In this case, the
    // // midInside bit is 1 if data[0] > data[1].
    // // Note that the bezier indices are already sorted from smallest to
    // // largest because of std::set.
    // if (midInside) {
    // 	// If cell is empty, there's nothing to swap (both values 0).
    // 	// So a fake "sort meta" value must be used to make data[0]
    // 	// be larger. This special value is treated as 0 by the shader.
    // 	if (beziers->size() == 0) {
    // 		data[0] = kBezierIndexSortMeta;
    // 	}
    // 	// If there's just one bezier, data[0] is always > data[1] so
    // 	// nothing needs to be done. Otherwise, swap data[0] and [1].
    // 	else if (beziers->size() != 1) {
    // 		uint8_t tmp = data[0];
    // 		data[0] = data[1];
    // 		data[1] = tmp;
    // 	}
    // // If midInside is 0, make sure that data[0] <= data[1]. This can only
    // // not happen if there is only 1 bezier in this cell, for the reason
    // // described above. Solve by moving the only bezier into data[1].
    // } else if (beziers->size() == 1) {
    // 	data[1] = data[0];
    // 	data[0] = kBezierIndexUnused;
    // }
}
