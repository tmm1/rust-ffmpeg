pub mod id;
pub use self::id::Id;

pub mod packet;

pub mod subtitle;

pub mod picture;

pub mod discard;

pub mod context;
pub use self::context::Context;

pub mod codec;

pub mod field_order;
pub mod audio_service;

pub mod compliance;
pub use self::compliance::Compliance;

pub mod debug;
pub use self::debug::Debug;

pub mod profile;
pub use self::profile::Profile;

pub mod threading;

pub mod encoder;
pub mod decoder;

use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;

pub fn version() -> u32 {
	unsafe {
		avcodec_version()
	}
}

pub fn configuration() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avcodec_configuration()).to_bytes())
	}
}

pub fn license() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avcodec_license()).to_bytes())
	}
}
