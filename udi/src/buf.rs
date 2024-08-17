//! Buffers (`udi_buf_t`)
//! 
//! 
use ::core::future::Future;
use crate::ffi::udi_buf_t;
use crate::ffi::buf::udi_buf_path_t;
use crate::ffi::buf::udi_buf_tag_t;
use crate::ffi::buf::udi_tagtype_t;

/// An owning buffer handle
#[repr(transparent)]
pub struct Handle(*mut udi_buf_t);

#[repr(transparent)]
/// A buffer path, used to determine what devices will be involved in handling a buffer.
pub struct Path(udi_buf_path_t);

impl Default for Handle {
    fn default() -> Self {
        Self(::core::ptr::null_mut())
    }
}
impl Default for Path {
    fn default() -> Self {
        Self(crate::ffi::buf::UDI_NULL_PATH_BUF)
    }
}

impl Handle
{
    /// Create a buffer handle from a pre-existing raw `*mut udi_buf_t` - mutable
    /// 
    /// UNSAFE: Caller must ensure either ownership or mutable access to `raw` (it can be null)
    pub unsafe fn from_ref(raw: &*mut udi_buf_t) -> &Self {
        &*(raw as *const _ as *const Self)
    }
    /// Create a buffer handle from a pre-existing raw `*mut udi_buf_t` - shared
    /// 
    /// UNSAFE: Caller must ensure either ownership or mutable access to `raw` (it can be null)
    pub unsafe fn from_mut(raw: &mut *mut udi_buf_t) -> &mut Self {
        &mut *(raw as *mut _ as *mut Self)
    }
    /// Construct a buffer handle from a raw pointer
    /// 
    /// UNSAFE: Caller must ensure either ownership or mutable access to `raw` (it can be null)
    pub unsafe fn from_raw(raw: *mut udi_buf_t) -> Self {
        Self(raw)
    }
    /// Steal a buffer pointer from some other location
    /// 
    /// UNSAFE: Caller must ensure either ownership or mutable access to `raw` (it can be null)
    pub unsafe fn take_raw(raw: &mut *mut udi_buf_t) -> Self {
        Self(::core::ptr::replace(raw, ::core::ptr::null_mut()))
    }
    /// Update this handle from a raw pointer
    /// 
    /// UNSAFE: Caller must ensure either ownership or mutable access to `raw` (it can be null)
    pub unsafe fn update_from_raw(&mut self, raw: *mut udi_buf_t) {
        self.0 = raw;
    }
    /// Obtain the raw pointer
    pub fn to_raw(&mut self) -> *mut udi_buf_t {
        self.0
    }
    /// Obtain the raw pointer (moving)
    pub fn into_raw(self) -> *mut udi_buf_t {
        let Handle(rv) = self;
        rv
    }

    /// Get an inclusive range from any range operator
    pub fn get_range(&self, range: impl ::core::ops::RangeBounds<usize>) -> ::core::ops::Range<usize> {
        use ::core::ops::Bound;
        let end_exl = match range.end_bound() {
            Bound::Excluded(&v) => v,
            Bound::Included(&v) => v + 1,
            Bound::Unbounded => self.len(),
            };
        let begin_incl = match range.start_bound() {
            Bound::Excluded(&v) => v + 1,
            Bound::Included(&v) => v,
            Bound::Unbounded => 0,
            };
        begin_incl .. end_exl
    }

    /// Get the buffer length
    pub fn len(&self) -> usize {
        if self.0.is_null() {
            0
        }
        else {
            // SAFE: Non-null and owned
            unsafe { (*self.0).buf_size }
        }
    }

    /// Construct a new buffer using provided data
    pub fn new<'d>(
    	cb: crate::CbRef<crate::ffi::udi_cb_t>,
        init_data: &'d [u8],
        path_handle: crate::ffi::buf::udi_buf_path_t
    ) -> impl Future<Output=Self> + 'd {
        crate::async_trickery::wait_task::<_, _,_,_>(
            cb,
            move |cb| unsafe {
                crate::ffi::buf::UDI_BUF_ALLOC(Self::callback, cb as *const _ as *mut _, init_data.as_ptr() as *const _, init_data.len(), path_handle)
                },
            |res| {
                let crate::WaitRes::Pointer(p) = res else { panic!(""); };
                // SAFE: Trusting the environemnt to have given us a valid pointer
                unsafe { Self::from_raw(p as *mut _) }
                }
            )
    }
    /// Ensure that this buffer has at least `size` bytes allocated within it
    /// 
    /// If the current size is smaller than `size`, then extra uninitialied bytes are added to the end
    pub fn ensure_size(&mut self, cb: crate::CbRef<crate::ffi::udi_cb_t>, size: usize) -> impl Future<Output=()> + '_
    {
        let self_buf = self.0;
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            cb,
            move |gcb| unsafe {
                let cur_size = if self_buf.is_null() { 0 } else { (*self_buf).buf_size };
                if cur_size >= size {
                    // Sufficient size
                    Self::callback(gcb, self_buf)
                }
                else {
                    let dst_off = cur_size;
                    let size = size - dst_off;
                    crate::ffi::buf::udi_buf_write(
                        Self::callback, gcb,
                        ::core::ptr::null(), size as _,
                        self_buf,
                        dst_off, 0,
                        crate::ffi::buf::UDI_NULL_PATH_BUF
                        );
                }
                },
            self.cb_update()
            )
    }
    /// Truncate the buffer's length to the specified value
    /// 
    /// NOTE: This doesn't do any environment calls, just sets `buf_size`
    pub fn truncate(&mut self, len: usize)
    {
        // SAFE: Logically owned
        unsafe {
            assert!(len <= self.len());
            if len != self.len() {
                (*self.0).buf_size = len;
            }
        }
    }

    /// Copy from one buffer to another (including tags)
    /// 
    /// - `src_range` - Source data range
    /// - `dst_range` - Destination range, if this is not the same size as `src_range` data will be shifted
    pub fn copy_from<'a>(
        &'a mut self,
        cb: crate::CbRef<crate::ffi::udi_cb_t>,
        src: &'a Handle,
        src_range: impl ::core::ops::RangeBounds<usize>,
        dst_range: impl ::core::ops::RangeBounds<usize>,
    ) -> impl ::core::future::Future<Output=()> + 'a
    {
        let src_range = src.get_range(src_range);
        let dst_range = self.get_range(dst_range);
        let self_buf = self.0;
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            cb,
            move |gcb| unsafe {
                crate::ffi::buf::udi_buf_copy(
                    Self::callback, gcb,
                    src.0, src_range.start, src_range.end - src_range.start,
                    self_buf, dst_range.start, dst_range.end - dst_range.start,
                    crate::ffi::buf::UDI_NULL_PATH_BUF
                    );
                },
            self.cb_update()
            )
    }

    /// Write data into a buffer
    pub fn write<'a>(&'a mut self,
    	cb: crate::CbRef<crate::ffi::udi_cb_t>,
        dst: impl ::core::ops::RangeBounds<usize>,
        data: &'a [u8]
    ) -> impl Future<Output=()> + 'a {
        let dst = self.get_range(dst);
        let self_buf = self.0;
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            cb,
            move |gcb| unsafe {
                crate::ffi::buf::udi_buf_write(
                    Self::callback, gcb,
                    data.as_ptr() as *const _, data.len(),
                    self_buf,
                    dst.start, dst.end - dst.start,
                    crate::ffi::buf::UDI_NULL_PATH_BUF
                    );
                },
            self.cb_update()
            )
    }

    /// Read data from a buffer into a slice
    pub fn read(&self, ofs: usize, dst: &mut [u8]) {
        assert!(ofs <= self.len());
        assert!(ofs + dst.len() <= self.len());
        // SAFE: Correct FFI inputs
        unsafe {
            crate::ffi::buf::udi_buf_read(self.0, ofs, dst.len(), dst.as_mut_ptr() as _)
        }
    }

    /// Consume and free this buffer
    pub fn free(mut self)
    {
        if !self.0.is_null() {
            unsafe { crate::ffi::buf::udi_buf_free(self.0); }
            self.0 = ::core::ptr::null_mut();
        }
    }

    unsafe extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, handle: *mut udi_buf_t) {
        unsafe { crate::async_trickery::signal_waiter(gcb, crate::WaitRes::Pointer(handle as *mut ())); }
    }
    fn cb_update(&mut self) -> impl FnOnce(crate::async_trickery::WaitRes) + '_ {
        move |res| {
            let crate::WaitRes::Pointer(p) = res else { panic!(""); };
            // SAFE: Trusting the environemnt to have given us a valid pointer
            unsafe { self.update_from_raw(p as *mut _); }
            }
    }
}
impl Handle
{
    /// Determine which of `path_handles` would give the best performance for allocating new buffers
    pub fn best_path_buf(&self, path_handles: &[Path], best_fit_array: &mut [u8], last_fit: usize) {
        assert!(path_handles.len() <= u8::MAX as usize);
        assert!(path_handles.len() == best_fit_array.len());
        assert!(last_fit < best_fit_array.len());
        // SAFE: Valid pointers (although being a little fast and loose with mutability)
        unsafe {
            crate::ffi::buf::udi_buf_best_path(self.0, 
                path_handles.as_ptr() as *const _ as *mut _, path_handles.len() as u8,
                last_fit as u8, best_fit_array.as_mut_ptr()
            );
        }
    }
}
/// Value tags - associated data outside of the buffer itself
impl Handle
{
    /// Set a collection of tags in the buffer
    pub fn tag_set<'a>(&'a mut self, cb: crate::CbRef<crate::ffi::udi_cb_t>, tags: &'a [udi_buf_tag_t]) -> impl Future<Output=()>+'a {
        let self_buf = self.0;
        #[cfg(debug_assertions)]
        for t in tags {
            debug_assert!(t.tag_off+t.tag_len <= self.len());
            debug_assert!(t.tag_type.count_ones() == 1);
        }
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            cb,
            move |gcb| unsafe {
                crate::ffi::buf::udi_buf_tag_set(
                    Self::callback, gcb,
                    self_buf,
                    tags.as_ptr() as *mut _, tags.len() as u16
                    );
                },
            self.cb_update()
            )
    }
    /// Get a collection of tags from the buffer
    pub fn tag_get<'a>(&self, tag_type_mask: udi_tagtype_t, dst: &'a mut [udi_buf_tag_t], skip: usize) -> &'a mut [udi_buf_tag_t] {
        let len = unsafe {
            crate::ffi::buf::udi_buf_tag_get(self.0, tag_type_mask, dst.as_mut_ptr(), dst.len() as u16, skip as u16)
        };
        &mut dst[..len as usize]
    }
    /// Compute a particular computable tag on the buffer range, and return the value
    pub fn tag_compute(&mut self, range: impl ::core::ops::RangeBounds<usize>, tag_type: udi_tagtype_t) -> crate::ffi::udi_ubit32_t {
        let range = self.get_range(range);
        debug_assert!(tag_type.count_ones() == 1);
        debug_assert!(tag_type & crate::ffi::buf::UDI_BUFTAG_VALUES != 0);
        let off = range.start;
        let len = range.end - range.start;
        unsafe {
            crate::ffi::buf::udi_buf_tag_compute(self.0, off, len, tag_type)
        }
    }
    /// Apply/update computable tags on a buffer
    pub fn tag_apply<'a>(&'a mut self, cb: crate::CbRef<crate::ffi::udi_cb_t>, tag_types_mask: udi_tagtype_t) -> impl Future<Output=()>+'a {
        debug_assert!(tag_types_mask & crate::ffi::buf::UDI_BUFTAG_UPDATES != 0);

        let self_buf = self.0;
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            cb,
            move |gcb| unsafe {
                crate::ffi::buf::udi_buf_tag_apply(Self::callback, gcb, self_buf, tag_types_mask);
                },
            self.cb_update()
            )
    }
}

impl Path
{
    /// Create a new path handle
    pub fn new(gcb: crate::CbRef<crate::ffi::udi_cb_t>) -> impl Future<Output=Path> {
        unsafe extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, handle: udi_buf_path_t) {
            // SAFE: Private function, gcb assumed valid
            unsafe { crate::async_trickery::signal_waiter(gcb, crate::WaitRes::Pointer(handle as *mut ())); }
        }

        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            gcb,
            move |gcb| unsafe {
                crate::ffi::buf::udi_buf_path_alloc(callback, gcb)
            },
            move |res| {
                let crate::async_trickery::WaitRes::Pointer(v) = res else { panic!() };
                Path(v as udi_buf_path_t)
            }
        )
    }
}
impl Drop for Path {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::buf::udi_buf_path_free(::core::mem::replace(&mut self.0, crate::ffi::buf::UDI_NULL_PATH_BUF));
        }
    }
}