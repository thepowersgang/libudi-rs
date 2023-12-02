use ::udi_sys as ffi;

pub type Result<T> = ::core::result::Result<T,Error>;

/// A wrapper around `udi_status_t` that cannot be `UDI_OK`
#[derive(Copy,Clone)]
pub struct Error(::core::num::NonZeroU32);
impl Error {
	pub fn into_inner(self) -> ffi::udi_status_t {
		self.0.get()
	}
	pub fn from_status(s: ffi::udi_status_t) -> Result<()> {
		match ::core::num::NonZeroU32::new(s) {
		Some(v) => Err(Error(v)),
		None => Ok( () ),
		}
	}
	pub fn to_status(r: Result<()>) -> ffi::udi_status_t {
		match r {
		Ok(()) => ffi::UDI_OK as _,
		Err(e) => e.into_inner(),
		}
	}
	pub fn as_str(&self) -> Option<&str> {
		macro_rules! v {
			( $($name:ident) *) => {
				$(const $name: u32 = ffi::$name as u32;)*
				Some(match self.0.get() {
				$($name => stringify!($name),)*
				_ => return None,
				})
			};
		}
		v!{
			UDI_STAT_NOT_SUPPORTED    
			UDI_STAT_NOT_UNDERSTOOD   
			UDI_STAT_INVALID_STATE    
			UDI_STAT_MISTAKEN_IDENTITY
			UDI_STAT_ABORTED          
			UDI_STAT_TIMEOUT          
			UDI_STAT_BUSY             
			UDI_STAT_RESOURCE_UNAVAIL 
			UDI_STAT_HW_PROBLEM       
			UDI_STAT_NOT_RESPONDING   
			UDI_STAT_DATA_UNDERRUN    
			UDI_STAT_DATA_OVERRUN     
			UDI_STAT_DATA_ERROR       
			UDI_STAT_PARENT_DRV_ERROR 
			UDI_STAT_CANNOT_BIND      
			UDI_STAT_CANNOT_BIND_EXCL 
			UDI_STAT_TOO_MANY_PARENTS 
			UDI_STAT_BAD_PARENT_TYPE  
			UDI_STAT_TERMINATED       
			UDI_STAT_ATTR_MISMATCH    
		}
	}
}
impl ::core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		if let Some(v) = self.as_str() {
			f.write_str(v)
		}
		else {
			write!(f, "{}", self.0.get())
		}
    }
}