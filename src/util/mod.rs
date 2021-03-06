pub mod dictionary;
pub mod error;
pub mod rational;
pub mod media;
pub mod picture;
pub mod color;
pub mod format;
pub mod frame;
pub mod chroma;
pub mod time;
pub mod channel_layout;

use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;

pub fn version() -> u32 {
	unsafe {
		avutil_version()
	}
}

pub fn configuration() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avutil_configuration()).to_bytes())
	}
}

pub fn license() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avutil_license()).to_bytes())
	}
}
