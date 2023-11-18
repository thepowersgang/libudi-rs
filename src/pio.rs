//! Programmed IO
//! 
//! A feature that allows doing register IO using a simple register-based VM
//! instead of needing drivers to run with direct IO access.

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
/// Map (register) a set of PIO operations and registers
/// 
/// - `regset`: Register set index (see the bus documentation)
/// - `offset` and `length` are the address region within the regset
/// - `pio_attributes`: 
/// - `pace_us`: Minimum duration between two IO accesses (microseconds)
/// - `serialization_domain`: All accesses to the same device with the same domain will be serialised (won't be interleaved)
pub fn map(
	cb: crate::CbRef<crate::ffi::udi_cb_t>,
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
	// TODO: Is there a way to call the FFI function outside of the future?
	// - When this is run, we're already in execution - so should be able to get the gcb
	// - Currently, the `start` callback cpatures all arguments, so is quite large (40 bytes or so?)
	crate::async_trickery::wait_task::<_, _,_,_>(
		cb,
		move |gcb| unsafe {
			crate::ffi::pio::udi_pio_map(
				cb_pio_map, gcb,
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

/// An unsafe-to-construct pointer used for [trans]'s `mem_ptr` argument
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

//// Execute a PIO transaction
/// 
/// - `buf` is a buffer usable by PIO transactions
/// - `mem_ptr` Memory block used by `UDI_PIO_MEM` transactions
pub fn trans<'a>(
	cb: crate::CbRef<crate::ffi::udi_cb_t>,
	pio_handle: &'a Handle,
	start_label: crate::ffi::udi_index_t,
	mut buf: Option<&'a mut crate::buf::Handle>,
	mem_ptr: Option<MemPtr<'a>>
	) -> impl ::core::future::Future<Output=Result<u16,crate::Error>> + 'a {
	extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, new_buf: *mut crate::ffi::udi_buf_t, status: crate::ffi::udi_status_t, result: u16) {
		unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Data([new_buf as usize, status as usize, result as usize, 0])); }
	}
	let buf_ptr = match buf { Some(ref mut v) => v.to_raw(), None => ::core::ptr::null_mut() };
	crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
		cb,
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
			let crate::WaitRes::Data([new_buf, status, result, ..]) = res else { panic!(""); };
			if let Some(buf) = buf {
				// SAFE: Trusting the environment
				unsafe { buf.update_from_raw(new_buf as *mut _); }
			}
			crate::Error::from_status(status as crate::ffi::udi_status_t).map(|()| result as u16)
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

		#[allow(non_snake_case)]
		pub mod B {
			pub const fn to_u16(v: u8) -> u16 {
				v as u16
			}
		}
		#[allow(non_snake_case)]
		pub mod S {
			pub const fn to_u16(v: u16) -> u16 {
				v
			}
		}
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
		pub const R2: u8 = 2;
		pub const R3: u8 = 3;
		pub const R4: u8 = 4;
		pub const R5: u8 = 5;
		pub const R6: u8 = 6;
		pub const R7: u8 = 7;
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
		pub const OR         : u8 = 0xC0;
		pub const OR_IMM     : u8 = 0xC8;
		pub const XOR        : u8 = 0xD0;
		pub const ADD        : u8 = 0xD8;
		pub const ADD_IMM    : u8 = 0xE0;
		pub const SUB        : u8 = 0xE8;
	}
	pub mod ops_group_c {
		pub const BRANCH    : u8 = 0xF0;
		pub const LABEL     : u8 = 0xF1;
		pub const REP_IN_IND: u8 = 0xF2;
		pub const REP_OUT_IND: u8 = 0xF3;
		/// Delay for at least `operand` microseconds
		pub const DELAY   : u8 = 0xF4;
		pub const BARRIER : u8 = 0xF5;
		pub const SYNC    : u8 = 0xF6;
		pub const SYNC_OUT: u8 = 0xF7;
		pub const DEBUG   : u8 = 0xF8;
		// Unallocated
		pub const END    : u8 = 0xFE;
		pub const END_IMM: u8 = 0xFF;
	}
}

/// Define a set of PIO operations
/// 
/// This macro implements a domain-specific syntax for PIO transation operations
/// 
/// Definitions:
/// - `size`: An operation size code. B = byte (`u8`), S = short (`u16`), `L` = long (`u32`), `_8` = `u64`, `_16` = `u128`, `_32` = 32 bytes)
/// - `Rsomething`: A register name, `R0` through to `R7`
/// - `regaddr`: A device register address
/// - `stride`: A memory stride distance, `STEP1`, `STEP2`, or `STEP4`. Strides are in multiples of the operation size
///
/// Commands:
/// - `IN.size Rd, regaddr` - Read from IO
/// - `OUT.size regaddr, Rs` - Write to IO
/// - ...
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

	// Group A
	// - IN Rd, reg
	(@expand $($output:expr,)*; IN.$sizecode:ident $reg:tt, $src:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@a $sizecode, IN, $reg, $src), ;
		$($rest)*
	) };
	// - OUT reg, Rs
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

	// Group B
	// - LOAD_IMM.[BS] Rd, IMM
	(@expand $($output:expr,)*; LOAD_IMM.$sizecode:ident $reg:ident, $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, LOAD_IMM, $reg, $crate::pio::vals::size::$sizecode::to_u16($val)), ;
		$($rest)*
	) };
	// - CSKIP.s Rt, cc
	(@expand $($output:expr,)*; CSKIP.$sizecode:ident $reg:ident $cc:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, CSKIP, $reg, $crate::pio::vals::ConditionCode::$cc as _), ;
		$($rest)*
	) };
	// - IN_IND
	(@expand $($output:expr,)*; IN_IND.$sizecode:ident $reg:ident, $pio_reg:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, IN_IND, $reg, $crate::pio::vals::regs::$pio_reg as _), ;
		$($rest)*
	) };
	// - OUT_IND
	(@expand $($output:expr,)*; OUT_IND.$sizecode:ident $pio_reg:ident, $reg:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, OUT_IND, $reg, $crate::pio::vals::regs::$pio_reg as _), ;
		$($rest)*
	) };
	// - SHIFT_LEFT.s Rd, bits
	(@expand $($output:expr,)*; SHIFT_LEFT.$sizecode:ident $reg:ident, $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, SHIFT_LEFT, $reg, $val), ;
		$($rest)*
	) };
	// - SHIFT_RIGHT.s Rd, bits
	(@expand $($output:expr,)*; SHIFT_RIGHT.$sizecode:ident $reg:ident, $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, SHIFT_RIGHT, $reg, $val), ;
		$($rest)*
	) };
	// - AND.s Rd, Rs
	(@expand $($output:expr,)*; AND.$sizecode:ident $reg:ident, $rs:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, AND, $reg, $crate::pio::vals::regs::$rs as _), ;
		$($rest)*
	) };
	// - AND_IMM Rd, IMM
	(@expand $($output:expr,)*; AND_IMM.$sizecode:ident $reg:ident, $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, AND_IMM, $reg, $crate::pio::vals::size::$sizecode::to_u16($val)), ;
		$($rest)*
	) };
	// - OR.s Rd, Rs
	(@expand $($output:expr,)*; OR.$sizecode:ident $reg:ident, $rs:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, OR, $reg, $crate::pio::vals::regs::$rs as _), ;
		$($rest)*
	) };
	// - OR_IMM.s Rd, IMM
	(@expand $($output:expr,)*; OR_IMM.$sizecode:ident $reg:ident, $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, OR_IMM, $reg, $crate::pio::vals::size::$sizecode::to_u16($val)), ;
		$($rest)*
	) };
	// - XOR.s Rd, Rs
	(@expand $($output:expr,)*; XOR.$sizecode:ident $reg:ident, $rs:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, XOR, $reg, $crate::pio::vals::regs::$rs as _), ;
		$($rest)*
	) };
	// - ADD.s Rd, Rs
	(@expand $($output:expr,)*; ADD.$sizecode:ident $reg:ident, $rs:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, ADD, $reg, $crate::pio::vals::regs::$rs as _), ;
		$($rest)*
	) };
	// - ADD_IMM.s Rd, IMM
	(@expand $($output:expr,)*; ADD_IMM.$sizecode:ident $reg:ident, $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, ADD_IMM, $reg, $crate::pio::vals::size::$sizecode::to_u16($val)), ;
		$($rest)*
	) };
	// - SUB.s Rd, Rs
	(@expand $($output:expr,)*; SUB.$sizecode:ident $reg:ident, $rs:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@b $sizecode, SUB, $reg, $crate::pio::vals::regs::$rs as _), ;
		$($rest)*
	) };

	// Group C
	// - BRANCH idx
	(@expand $($output:expr,)*; BRANCH $idx:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, BRANCH, $idx), ;
		$($rest)*
	) };
	// - LABEL idx
	(@expand $($output:expr,)*; LABEL $idx:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, LABEL, $idx), ;
		$($rest)*
	) };
	// REP_IN_IND [ {mem,buf} Rmem {|stride} ], Rreg {|stride}, Rcount
	(@expand $($output:expr,)*;
		REP_IN_IND.$sizecode:ident [$ty:ident $mem_reg:ident $($mem_stride:ident)?], $pio_reg:ident $($pio_stride:ident)?, $count_reg:ident;
		$($rest:tt)*
	) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, REP_IN_IND, $crate::define_pio_ops!(@rep_args $ty $mem_reg $($mem_stride)?, $pio_reg $($pio_stride)?, $count_reg)), ;
		$($rest)*
	) };
	// REP_OUT_IND.s [ [mem|buf] Rmem [stride]], Rreg [stride], Rcount
	(@expand $($output:expr,)*;
		REP_OUT_IND.$sizecode:ident [$ty:ident $mem_reg:ident $($mem_stride:ident)?], $pio_reg:ident $($pio_stride:ident)?, $count_reg:ident;
		$($rest:tt)*
	) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, REP_OUT_IND, $crate::define_pio_ops!(@rep_args $ty $mem_reg $($mem_stride)?, $pio_reg $($pio_stride)?, $count_reg)), ;
		$($rest)*
	) };

	// `END.[BS] Rn` - 
	(@expand $($output:expr,)*; END.B $reg:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, END, $crate::pio::vals::regs::$reg as _), ;
		$($rest)*
	) };
	(@expand $($output:expr,)*; END.S $reg:ident; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c S, END, $crate::pio::vals::regs::$reg as _), ;
		$($rest)*
	) };
	// `DELAY microseconds` - Delay for AT LEAST `microseconds`
	(@expand $($output:expr,)*; DELAY $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c B, DELAY, $val), ;
		$($rest)*
	) };
	// `END imm` - 
	(@expand $($output:expr,)*; END_IMM $val:expr; $($rest:tt)* ) => { $crate::define_pio_ops!(@expand
		$($output,)* $crate::define_pio_ops!(@c S, END_IMM, $val), ;
		$($rest)*
	) };

	// ----- Encoding -----
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
			pio_op: $crate::pio::vals::ops_group_b::$opname|$crate::pio::vals::regs::$regname|$crate::ffi::pio::UDI_PIO_DIRECT,
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

	// ----- Arguments for the repeat ops -----
	(@rep_args mem $mem_reg:ident $($mem_stride:ident)?, $pio_reg:ident $($pio_stride:ident)?, $count_reg:ident) => {
		$crate::ffi::pio::UDI_PIO_MEM as u16
		|($crate::pio::vals::regs::$mem_reg as u16)
		$(| $crate::pio::vals::stride::$mem_stride << 5)?
		|($crate::pio::vals::regs::$pio_reg as u16) << 7
		$(| $crate::pio::vals::stride::$pio_stride << 10)?
		|($crate::pio::vals::regs::$count_reg as u16) << 12
	};
	(@rep_args buf $mem_reg:ident $($mem_stride:ident)?, $pio_reg:ident $($pio_stride:ident)?, $count_reg:ident) => {
		$crate::ffi::pio::UDI_PIO_BUF as u16
		|($crate::pio::vals::regs::$mem_reg as u16)
		$(| $crate::pio::vals::stride::$mem_stride << 5)?
		|($crate::pio::vals::regs::$pio_reg as u16) << 7
		$(| $crate::pio::vals::stride::$pio_stride << 10)?
		|($crate::pio::vals::regs::$count_reg as u16) << 12
	};
	(@rep_args scratch $mem_reg:ident $($mem_stride:ident)?, $pio_reg:ident $($pio_stride:ident)?, $count_reg:ident) => {
		$crate::ffi::pio::UDI_PIO_SCRATCH as u16
		|($crate::pio::vals::regs::$mem_reg as u16)
		$(| $crate::pio::vals::stride::$mem_stride << 5)?
		|($crate::pio::vals::regs::$pio_reg as u16) << 7
		$(| $crate::pio::vals::stride::$pio_stride << 10)?
		|($crate::pio::vals::regs::$count_reg as u16) << 12
	};
}