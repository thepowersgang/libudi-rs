use ::udi::ffi::buf::udi_buf_copy_call_t;
use ::udi::ffi::buf::udi_buf_path_t;
use ::udi::ffi::udi_buf_t;
use ::udi::ffi::udi_cb_t;
use ::udi::ffi::udi_size_t;
use ::udi::ffi::c_void;

#[repr(C)]
struct RealUdiBuf {
    raw: udi_buf_t,
    inner: Vec<u8>,
}

impl RealUdiBuf {
    fn len(&self) -> usize {
        assert!(self.raw.buf_size <= self.inner.len());
        usize::min(self.raw.buf_size, self.inner.len())
    }
    fn get_slice(&self, off: udi_size_t, len: udi_size_t) -> &[u8] {
        let max_len = self.len();
        let slice = &self.inner[..max_len];
        let slice = &slice[off as usize..];
        &slice[..len]
    }
    fn get_slice_mut(&mut self, off: udi_size_t, len: udi_size_t) -> &mut [u8] {
        let max_len = self.len();
        let slice = &mut self.inner[..max_len];
        let slice = &mut slice[off as usize..];
        &mut slice[..len]
    }

    fn delete_at(&mut self, off: udi_size_t, count: udi_size_t) {
        assert!(off <= self.len());
        assert!(count <= self.len());
        assert!(off+count <= self.len());
        self.inner.copy_within(off+count.., count);
        let new_len = self.len() - count;
        self.inner.truncate(new_len);
        self.raw.buf_size = new_len;
    }
    fn reserve_at(&mut self, off: udi_size_t, count: udi_size_t) {
        assert!(off <= self.len());
        let old_len = self.len();
        self.inner.resize(count, 0);
        self.inner.copy_within(off..old_len, off+count);
        self.raw.buf_size = self.inner.len();
    }
}
unsafe fn get_buf_mut(ptr: &mut *mut udi_buf_t) -> &mut RealUdiBuf {
    if ptr.is_null() {
        *ptr = Box::into_raw(Box::new(RealUdiBuf { raw: udi_buf_t { buf_size: 0 }, inner: vec![], })) as _;
    }
    &mut *(*ptr as *mut RealUdiBuf)
}
unsafe fn get_buf(ptr: &*mut udi_buf_t) -> Option<&RealUdiBuf> {
    if ptr.is_null() {
        None
    }
    else {
        Some( &*(*ptr as *mut RealUdiBuf) )
    }
}

#[no_mangle]
unsafe extern "C" fn udi_buf_copy(
    callback: udi_buf_copy_call_t,
    gcb: *mut udi_cb_t,
    src_buf: *mut udi_buf_t,
    src_off: udi_size_t,
    src_len: udi_size_t,
    dst_buf: *mut udi_buf_t,
    dst_off: udi_size_t,
    dst_len: udi_size_t,
    _path_handle: udi_buf_path_t
)
{
    assert!(src_buf != dst_buf, "Not allowed to reference the same buffer");
    let src = get_buf(&src_buf).unwrap().get_slice(src_off, src_len);
    udi_buf_write(callback, gcb, src.as_ptr() as *const c_void, src.len(), dst_buf, dst_off, dst_len, _path_handle);
}
/// [udi_buf_copy] logically replaces dst_len bytes of data starting at offset
/// dst_offset in dst_buf with a copy of src_len bytes of data starting at
/// src_offset in src_buf. When the data has been copied, the callback
/// routine is called.
#[no_mangle]
unsafe extern "C" fn udi_buf_write(
    callback: udi_buf_copy_call_t,
    gcb: *mut udi_cb_t,
    src_buf: *const c_void,
    src_len: udi_size_t,
    mut dst_buf: *mut udi_buf_t,
    dst_off: udi_size_t,
    dst_len: udi_size_t,
    _path_handle: udi_buf_path_t
)
{
    let dst = get_buf_mut(&mut dst_buf);
    if src_buf.is_null() {
        // If `src_buf` is NULL, then the data is unspecified
        if src_len == dst_len {
        }
        else if src_len < dst_len {
            // Delete
            dst.delete_at(dst_off+src_len, dst_len - src_len);
        }
        else {
            // Reserve
            dst.reserve_at(dst_off + dst_len, src_len - dst_len);
        }
    }
    else {
        let src = if src_len == 0 {
            b""
        }
        else {
            ::core::slice::from_raw_parts(src_buf as *mut u8, src_len)
        };

        if src_len == dst_len {
        }
        else if src_len < dst_len {
            dst.delete_at(dst_off+src_len, dst_len - src_len);
        }
        else {
            dst.reserve_at(dst_off + dst_len, src_len - dst_len);
        }
        // Update the data
        let min_len = usize::min(dst_len, src_len);
        dst.get_slice_mut(dst_off, min_len).copy_from_slice(&src[..min_len]);
    }
    
    callback(gcb, dst_buf);
}
#[no_mangle]
unsafe extern "C" fn udi_buf_free(buf: *mut udi_buf_t)
{
    todo!()
}