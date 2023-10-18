#[derive(Debug)]
pub struct Handle(crate::ffi::pio::udi_pio_handle_t);
impl Handle {
	pub fn as_raw(&self) -> crate::ffi::pio::udi_pio_handle_t {
		self.0
	}
}
impl ::core::default::Default for Handle {
	fn default() -> Self {
		Handle(::core::ptr::null_mut())
	}
}
/// - `regset`: Register set index (see the bus documentation)
/// - `offset` and `length` are the address region within the regset
/// - `pio_attributes`: 
/// - `pace_us`: Minimum duration between two IO accesses (microseconds)
/// - `serialization_domain`: All accesses to the same device with the same domain will be serialised (won't be interleaved)
pub fn map(
	regset: u32,
	offset: u32, length: u32,
	trans_list: &'static [crate::ffi::pio::udi_pio_trans_t],
	pio_attributes: u16,
	pace_us: u32,
	serialization_domain: crate::ffi::udi_index_t,
) -> impl ::core::future::Future<Output=Handle>
{
	extern "C" fn cb_pio_map(gcb: *mut crate::ffi::udi_cb_t, handle: crate::ffi::pio::udi_pio_handle_t) {
		unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
	}
	crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
		move |cb| unsafe {
			crate::ffi::pio::udi_pio_map(
				cb_pio_map, cb as *const _ as *mut _,
				regset, offset, length, trans_list.as_ptr(), trans_list.len() as u16,
				pio_attributes, pace_us, serialization_domain
			)
			},
		|res| {
			let crate::WaitRes::Pointer(p) = res else { panic!(""); };
			Handle(p as *mut _)
			}
		)
}

pub struct MemPtr<'a>(&'a mut [u8]);
impl<'a> MemPtr<'a> {
	/// UNSAFE: There is no ability to bounds check this buffer, the PIO ops must not write out of range
	pub unsafe fn new(p: &'a mut [u8]) -> MemPtr<'a> {
		MemPtr(p)
	}
	fn as_raw(&mut self) -> *mut core::ffi::c_void {
		self.0.as_mut_ptr() as *mut _
	}
}

/// - `buf` is a buffer usable by PIO transactions
/// - `mem_ptr` Memory block used by `UDI_PIO_MEM` transactions
pub fn trans<'a>(
	pio_handle: &'a Handle,
	start_label: crate::ffi::udi_index_t,
	mut buf: Option<&'a mut crate::buf::Handle>,
	mem_ptr: Option<MemPtr<'a>>
	) -> impl ::core::future::Future<Output=Result<u16,crate::ffi::udi_status_t>> + 'a {
	extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, new_buf: *mut crate::ffi::udi_buf_t, status: crate::ffi::udi_status_t, result: u16) {
		unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Data([new_buf as usize, status as usize, result as usize])); }
	}
	let buf_ptr = match buf { Some(ref mut v) => v.to_raw(), None => ::core::ptr::null_mut() };
	crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
		move |cb| unsafe {
			crate::ffi::pio::udi_pio_trans(
				callback, cb as *const _ as *mut _,
				pio_handle.0,
				start_label,
				buf_ptr,
				match mem_ptr { Some(mut v) => v.as_raw(), None => ::core::ptr::null_mut() },
			)
			},
		|res| {
			let crate::WaitRes::Data([new_buf, status, result]) = res else { panic!(""); };
			if let Some(buf) = buf {
				// SAFE: Trusting the environment
				unsafe { buf.update_from_raw(new_buf as *mut _); }
			}
			if status != 0 { Err(status as crate::ffi::udi_status_t) } else { Ok(result as u16) }
			}
		)
}

#[doc(hidden)]
pub mod vals {
	pub const fn u8_to_u16(v: u8) -> u16 {
		v as u16
	}
	pub mod size {
		pub const B: u8 = crate::ffi::pio::UDI_PIO_1BYTE;
		pub const S: u8 = 1;//crate::ffi::pio::UDI_PIO_2BYTE;
		pub const L: u8 = 2;//crate::ffi::pio::UDI_PIO_4BYTE;
		pub const _8: u8 = 3;//crate::ffi::pio::UDI_PIO_8BYTE;
		pub const _16: u8 = 4;//crate::ffi::pio::UDI_PIO_16BYTE;
		pub const _32: u8 = 5;//crate::ffi::pio::UDI_PIO_32BYTE;
	}
	pub mod stride {
		pub const STEP1: u16 = 1;
		pub const STEP2: u16 = 2;
		pub const STEP4: u16 = 3;
	}
	#[repr(C)]
	pub enum ConditionCode {
		Z,
		NZ,
		Neg,
		NNeg,
	}
	pub mod regs {
		pub const R0: u8 = 0;
		pub const R1: u8 = 1;
	}
	// Group A operations: The register parameter can be a memory reference, or direct
	pub mod ops_group_a {
		pub const IN   : u8 = 0x00;
		pub const OUT  : u8 = 0x20;
		pub const LOAD : u8 = 0x40;
		pub const STORE: u8 = 0x60;
	}
	// Group B operations: Registers can only be direct
	pub mod ops_group_b {
		pub const LOAD_IMM   : u8 = 0x80;
		pub const CSKIP      : u8 = 0x88;
		pub const IN_IND     : u8 = 0x90;
		pub const OUT_IND    : u8 = 0x98;
		pub const SHIFT_LEFT : u8 = 0xA0;
		pub const SHIFT_RIGHT: u8 = 0xA8;
		pub const AND        : u8 = 0xB0;
		pub const AND_IMM    : u8 = 0xB8;
	}
	pub mod ops_group_c {
		pub const LABEL     : u8 = 0xF0;
		pub const BRANCH    : u8 = 0xF1;
		pub const REP_IN_IND: u8 = 0xF2;
		pub const END    : u8 = 0xFE;
		pub const END_IMM: u8 = 0xFF;
	}
}

#[macro_export]
macro_rules! define_pio_ops
{
	(
		$v:vis $name:ident =
		$($inner:tt)*
	) => {
		$v const $name: &'static [$crate::ffi::pio::udi_pio_trans_t] =
			&$crate::define_pio_ops!(@expand ; $($inner)*);
	};

	(@expand $($output:expr,)*; ) => { [ $($output,)* ] };

	(@expand $($output:expr,)*; IN.$sizecode:ident $reg:tt, $src:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@a $sizecode, IN, $reg, $src), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*; OUT.$sizecode:ident $dst:expr, $reg:tt; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@a $sizecode, OUT, $reg, $dst), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*; LOAD.$sizecode:ident $reg:ident, $src:tt; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@a $sizecode, LOAD, $src, $crate::pio::vals::regs::$reg as _), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*; STORE.$sizecode:ident $dst:tt, $reg:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@a $sizecode, STORE, $dst, $crate::pio::vals::regs::$reg as _), ;
		$($rest)*
	) };

	(@expand $($output:expr,)*; LOAD_IMM.B $reg:ident, $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b B, LOAD_IMM, $reg, $crate::pio::vals::u8_to_u16($val)), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*; LOAD_IMM.H $reg:ident, $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b H, LOAD_IMM, $reg, $val), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*; CSKIP.$sizecode:ident $reg:ident $cc:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, CSKIP, $reg, $crate::pio::vals::ConditionCode::$cc as _), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*; AND_IMM.$sizecode:ident $reg:tt, $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, AND_IMM, $reg, $val), ;
		$($rest)*
	) };

	(@expand $($output:expr,)*; BRANCH $idx:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, BRANCH, $idx), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*; LABEL $idx:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, LABEL, $idx), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*;
		REP_IN_IND.$sizecode:ident $ty:ident $mem_reg:ident $($mem_stride:ident)?, $pio_reg:ident $($pio_stride:ident)?, $count_reg:ident;
		$($rest:tt)*
	) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, REP_IN_IND, $crate::define_pio_ops!(@rep_args $ty $mem_reg $($mem_stride)?, $pio_reg $($pio_stride)?, $count_reg)), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*;
		REP_OUT_IND.$sizecode:ident $ty:ident $mem_reg:ident $($mem_stride:ident)?, $pio_reg:ident $($pio_stride:ident)?, $count_reg:ident;
		$($rest:tt)*
	) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, REP_OUT_IND, $crate::define_pio_ops!(@rep_args $ty $mem_reg $($mem_stride)?, $pio_reg $($pio_stride)?, $count_reg)), ;
		$($rest)*
	) };

	(@expand $($output:expr,)*; END.$sizecode:ident $reg:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c $sizecode, END, $crate::pio::vals::regs::$reg as _), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*; END_IMM $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, END_IMM, $val), ;
		$($rest)*
	) };

	// Group A
	(@a $size:ident, $opname:ident, $regname:ident, $val:expr) => {
		$crate::ffi::pio::udi_pio_trans_t {
			pio_op: $crate::pio::vals::ops_group_a::$opname|$crate::pio::vals::regs::$regname|$crate::ffi::pio::UDI_PIO_DIRECT,
			tran_size: $crate::pio::vals::size::$size,
			operand: $val
		}
		};
	// - Scratch could be unsafe, but interrupt handlers need it
	(@a $size:ident, $opname:ident, [scratch $regname:ident], $val:expr) => {
		$crate::ffi::pio::udi_pio_trans_t {
			pio_op: $crate::pio::vals::ops_group_a::$opname|$crate::pio::vals::regs::$regname|$crate::ffi::pio::UDI_PIO_SCRATCH,
			tran_size: $crate::pio::vals::size::$size,
			operand: $val
		}
	};
	(@a $size:ident, $opname:ident, [buf $regname:ident], $val:expr) => {
		$crate::ffi::pio::udi_pio_trans_t {
			pio_op: $crate::pio::vals::ops_group_a::$opname|$crate::pio::vals::regs::$regname|$crate::ffi::pio::UDI_PIO_BUF,
			tran_size: $crate::pio::vals::size::$size,
			operand: $val
		}
	};
	(@a $size:ident, $opname:ident, [mem $regname:ident], $val:expr) => {
		$crate::ffi::pio::udi_pio_trans_t {
			pio_op: $crate::pio::vals::ops_group_a::$opname|$crate::pio::vals::regs::$regname|$crate::ffi::pio::UDI_PIO_MEM,
			tran_size: $crate::pio::vals::size::$size,
			operand: $val
		}
	};
	
	// Group B
	(@b $size:ident, $opname:ident, $regname:ident, $val:expr) => {
		$crate::ffi::pio::udi_pio_trans_t {
			pio_op: $crate::pio::vals::ops_group_b::$opname|$crate::ffi::pio::UDI_PIO_DIRECT,
			tran_size: $crate::pio::vals::size::$size,
			operand: $val
		}
		};
	// Group C
	(@c $size:ident, $opname:ident, $val:expr) => {
		$crate::ffi::pio::udi_pio_trans_t {
			pio_op: $crate::pio::vals::ops_group_c::$opname,
			tran_size: $crate::pio::vals::size::$size,
			operand: $val
		}
		};

	(@count ($output:expr); ()) => { $output };
	(@count ($output:expr); (; $($rest:tt)*)) => { $crate::define_pio_ops!(@count ($output+1); ($($rest)*)) };
	(@count ($output:expr); ($t:tt $($rest:tt)*)) => { $crate::define_pio_ops!(@count ($output); ($($rest)*)) };

	(@rep_args mem $mem_reg:ident $($mem_stride:ident)?, $pio_reg:ident $($pio_stride:ident)?, $count_reg:ident) => {
		$crate::ffi::pio::UDI_PIO_MEM as u16
		|($crate::pio::vals::regs::$mem_reg as u16)
		$(| $crate::pio::vals::stride::$mem_stride << 5)?
	};
}