use libc::c_int;
use ffi::*;

bitflags! {
	flags Flags: c_int {
		const GRAY    = CODEC_FLAG_GRAY as c_int,
	}
}

