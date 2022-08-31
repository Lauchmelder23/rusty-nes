use std::ffi::c_void;

extern "C" 
{
	#[allow(improper_ctypes)]
	pub fn init_opengl(
		loader: *mut c_void, 
		f: fn(*mut c_void, *const i8) -> *const c_void
	) -> i32;

	pub fn clear();
}