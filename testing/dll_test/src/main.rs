use libc::{c_float, c_int, c_uint};
use std::ptr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::slice;

#[repr(C)]
struct XYZ {
    x: c_float,
    y: c_float,
    z: c_float,
}

#[link(name = "Navigation", kind = "dylib")]
extern "C" {
    fn CalculatePath(
        id: c_uint,
        start: XYZ,
        end: XYZ,
        smooth_path: c_int,
        path_length: *mut c_int,
    ) -> *mut XYZ;
}

fn main() {
    let start = XYZ {
        x: -10531.080078125,
        y: -1189.0,
        z: 28.0,
    };

    let end = XYZ {
        x: -10501.042025497547,
        y: -1185.1095881134734,
        z: 28.13749926004446,
    };

    let mut path_length: c_int = 0;

    unsafe {
        println!("calling function...");

        let path_ptr = CalculatePath(0, start, end, 0, &mut path_length);

        if !path_ptr.is_null() && path_length > 0 {
            println!("Path Length: {}", path_length);

            let path_slice = slice::from_raw_parts(path_ptr, path_length as usize);

            for (i, point) in path_slice.iter().enumerate() {
                println!(
                    "Point {}: X={}, Y={}, Z={}",
                    i, point.x, point.y, point.z
                );
            }
        } else {
            println!("Failed to calculate path or an error occurred.");
        }
    }

    println!("End.");
}

