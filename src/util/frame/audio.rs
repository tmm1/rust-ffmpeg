use std::mem;
use std::slice;
use std::ops::{Deref, DerefMut};

use libc::{c_int, int64_t, c_ulonglong};
use ffi::*;
use ::ChannelLayout;
use ::util::format;
use super::Frame;

#[derive(PartialEq, Eq)]
pub struct Audio(Frame);

impl Audio {
	#[inline(always)]
	pub unsafe fn wrap(ptr: *mut AVFrame) -> Self {
		Audio(Frame::wrap(ptr))
	}

	#[inline]
	pub unsafe fn alloc(&mut self, format: format::Sample, samples: usize, layout: ChannelLayout) {
		self.set_format(format);
		self.set_samples(samples);
		self.set_channel_layout(layout);

		av_frame_get_buffer(self.as_mut_ptr(), 1);
	}
}

impl Audio {
	#[inline(always)]
	pub fn empty() -> Self {
		unsafe {
			Audio(Frame::empty())
		}
	}

	#[inline]
	pub fn new(format: format::Sample, samples: usize, layout: ChannelLayout) -> Self {
		unsafe {
			let mut frame = Audio::empty();
			frame.alloc(format, samples, layout);

			frame
		}
	}

	#[inline]
	pub fn format(&self) -> format::Sample {
		unsafe {
			if (*self.as_ptr()).format == -1 {
				format::Sample::None
			}
			else {
				format::Sample::from(mem::transmute::<_, AVSampleFormat>(((*self.as_ptr()).format)))
			}
		}
	}

	#[inline]
	pub fn set_format(&mut self, value: format::Sample) {
		unsafe {
			(*self.as_mut_ptr()).format = mem::transmute::<AVSampleFormat, c_int>(value.into());
		}
	}

	#[inline]
	pub fn channel_layout(&self) -> ChannelLayout {
		unsafe {
			ChannelLayout::from_bits_truncate(av_frame_get_channel_layout(self.as_ptr()) as c_ulonglong)
		}
	}

	#[inline]
	pub fn set_channel_layout(&mut self, value: ChannelLayout) {
		unsafe {
			av_frame_set_channel_layout(self.as_mut_ptr(), value.bits() as int64_t);
		}
	}

	#[inline]
	pub fn channels(&self) -> u16 {
		unsafe {
			av_frame_get_channels(self.as_ptr()) as u16
		}
	}

	#[inline]
	pub fn set_channels(&mut self, value: u16) {
		unsafe {
			av_frame_set_channels(self.as_mut_ptr(), value as c_int);
		}
	}

	#[inline]
	pub fn rate(&self) -> u32 {
		unsafe {
			av_frame_get_sample_rate(self.as_ptr()) as u32
		}
	}

	#[inline]
	pub fn set_rate(&mut self, value: u32) {
		unsafe {
			av_frame_set_sample_rate(self.as_mut_ptr(), value as c_int);
		}
	}

	#[inline]
	pub fn samples(&self) -> usize {
		unsafe {
			(*self.as_ptr()).nb_samples as usize
		}
	}

	#[inline]
	pub fn set_samples(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).nb_samples = value as c_int;
		}
	}

	#[inline]
	pub fn is_planar(&self) -> bool {
		self.format().is_planar()
	}

	#[inline]
	pub fn is_packed(&self) -> bool {
		self.format().is_packed()
	}

	#[inline]
	pub fn planes(&self) -> usize {
		unsafe {
			if (*self.as_ptr()).linesize[0] == 0 {
				return 0;
			}
		}

		if self.is_packed() {
			1
		}
		else {
			self.channels() as usize
		}
	}

	#[inline]
	pub fn plane<T: Sample>(&self, index: usize) -> &[T] {
		if index >= self.planes() {
			panic!("out of bounds");
		}

		if !<T as Sample>::is_valid(self.format(), self.channels()) {
			panic!("unsupported type");
		}

		unsafe {
			slice::from_raw_parts(
				mem::transmute((*self.as_ptr()).data[index]),
				mem::size_of::<T>() * self.samples())
		}
	}

	#[inline]
	pub fn plane_mut<T: Sample>(&mut self, index: usize) -> &[T] {
		if index >= self.planes() {
			panic!("out of bounds");
		}

		if !<T as Sample>::is_valid(self.format(), self.channels()) {
			panic!("unsupported type");
		}

		unsafe {
			slice::from_raw_parts_mut(
				mem::transmute((*self.as_mut_ptr()).data[index]),
				mem::size_of::<T>() * self.samples())
		}
	}

	#[inline]
	pub fn data(&self) -> Vec<&[u8]> {
		let mut result = Vec::new();

		unsafe {
			for i in 0 .. self.planes() {
				result.push(slice::from_raw_parts(
					(*self.as_ptr()).data[i],
					(*self.as_ptr()).linesize[0] as usize));
			}
		}

		result
	}

	#[inline]
	pub fn data_mut(&mut self) -> Vec<&mut [u8]> {
		let mut result = Vec::new();

		unsafe {
			for i in 0 .. self.planes() {
				result.push(slice::from_raw_parts_mut(
					(*self.as_mut_ptr()).data[i],
					(*self.as_ptr()).linesize[0] as usize));
			}
		}

		result
	}
}

impl Deref for Audio {
	type Target = Frame;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Audio {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}

impl ::std::fmt::Debug for Audio {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
		try!(f.write_str("ffmpeg::frame::Audio { "));
		try!(f.write_str(&format!("format: {:?}, ", self.format())));
		try!(f.write_str(&format!("channels: {:?}, ", self.channels())));
		try!(f.write_str(&format!("rate: {:?}, ", self.rate())));
		try!(f.write_str(&format!("samples: {:?} ", self.samples())));
		f.write_str("}")
	}
}

impl Clone for Audio {
	fn clone(&self) -> Self {
		let mut cloned = Audio::new(self.format(), self.samples(), self.channel_layout());
		cloned.clone_from(self);

		cloned
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_frame_copy(self.as_mut_ptr(), source.as_ptr());
			av_frame_copy_props(self.as_mut_ptr(), source.as_ptr());
		}
	}
}

pub unsafe trait Sample {
	fn is_valid(format: format::Sample, channels: u16) -> bool;
}

unsafe impl Sample for u8 {
	#[inline(always)]
	fn is_valid(format: format::Sample, _channels: u16) -> bool {
		if let format::Sample::U8(..) = format {
			true
		}
		else {
			false
		}
	}
}

unsafe impl Sample for (u8, u8) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 2 && format == format::Sample::U8(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (u8, u8, u8) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 3 && format == format::Sample::U8(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (u8, u8, u8, u8) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 4 && format == format::Sample::U8(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (u8, u8, u8, u8, u8) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 5 && format == format::Sample::U8(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (u8, u8, u8, u8, u8, u8) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 6 && format == format::Sample::U8(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (u8, u8, u8, u8, u8, u8, u8) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 7 && format == format::Sample::U8(format::sample::Type::Packed)
	}
}

unsafe impl Sample for i16 {
	#[inline(always)]
	fn is_valid(format: format::Sample, _channels: u16) -> bool {
		if let format::Sample::I16(..) = format {
			true
		}
		else {
			false
		}
	}
}

unsafe impl Sample for (i16, i16) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 2 && format == format::Sample::I16(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i16, i16, i16) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 3 && format == format::Sample::I16(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i16, i16, i16, i16) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 4 && format == format::Sample::I16(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i16, i16, i16, i16, i16) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 5 && format == format::Sample::I16(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i16, i16, i16, i16, i16, i16) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 6 && format == format::Sample::I16(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i16, i16, i16, i16, i16, i16, i16) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 7 && format == format::Sample::I16(format::sample::Type::Packed)
	}
}

unsafe impl Sample for i32 {
	#[inline(always)]
	fn is_valid(format: format::Sample, _channels: u16) -> bool {
		if let format::Sample::I32(..) = format {
			true
		}
		else {
			false
		}
	}
}

unsafe impl Sample for (i32, i32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 2 && format == format::Sample::I32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i32, i32, i32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 3 && format == format::Sample::I32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i32, i32, i32, i32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 4 && format == format::Sample::I32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i32, i32, i32, i32, i32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 5 && format == format::Sample::I32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i32, i32, i32, i32, i32, i32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 6 && format == format::Sample::I32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (i32, i32, i32, i32, i32, i32, i32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 7 && format == format::Sample::I32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for f32 {
	#[inline(always)]
	fn is_valid(format: format::Sample, _channels: u16) -> bool {
		if let format::Sample::F32(..) = format {
			true
		}
		else {
			false
		}
	}
}

unsafe impl Sample for (f32, f32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 2 && format == format::Sample::F32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f32, f32, f32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 3 && format == format::Sample::F32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f32, f32, f32, f32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 4 && format == format::Sample::F32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f32, f32, f32, f32, f32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 5 && format == format::Sample::F32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f32, f32, f32, f32, f32, f32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 6 && format == format::Sample::F32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f32, f32, f32, f32, f32, f32, f32) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 7 && format == format::Sample::F32(format::sample::Type::Packed)
	}
}

unsafe impl Sample for f64 {
	#[inline(always)]
	fn is_valid(format: format::Sample, _channels: u16) -> bool {
		if let format::Sample::F64(..) = format {
			true
		}
		else {
			false
		}
	}
}

unsafe impl Sample for (f64, f64) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 2 && format == format::Sample::F64(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f64, f64, f64) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 3 && format == format::Sample::F64(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f64, f64, f64, f64) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 4 && format == format::Sample::F64(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f64, f64, f64, f64, f64) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 5 && format == format::Sample::F64(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f64, f64, f64, f64, f64, f64) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 6 && format == format::Sample::F64(format::sample::Type::Packed)
	}
}

unsafe impl Sample for (f64, f64, f64, f64, f64, f64, f64) {
	#[inline(always)]
	fn is_valid(format: format::Sample, channels: u16) -> bool {
		channels == 7 && format == format::Sample::F64(format::sample::Type::Packed)
	}
}
