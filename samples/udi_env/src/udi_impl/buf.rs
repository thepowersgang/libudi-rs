use ::udi::ffi::buf::udi_buf_copy_call_t;
use ::udi::ffi::buf::udi_buf_path_t;
use ::udi::ffi::buf::udi_buf_tag_t;
use ::udi::ffi::udi_buf_t;
use ::udi::ffi::udi_cb_t;
use ::udi::ffi::udi_size_t;
use ::udi::ffi::c_void;

#[repr(C)]
struct RealUdiBuf {
    raw: udi_buf_t,
    inner: Vec<u8>,
    tags: Vec<Tag>,

    path: udi_buf_path_t,
}
struct RealPath {
}
struct Tag {
    tag_type: u32,
    tag_value: u32,
    tag_off: usize,
    tag_len: usize,
}
impl Tag {
    fn get_type(&self, cur_driver_idx: u32) -> ::udi::ffi::buf::udi_tagtype_t {
        if self.tag_type >= 24 {
            let driver_idx = (self.tag_type - 24) / 8;
            if driver_idx != cur_driver_idx {
                0
            }
            else {
                1 << (24 + self.tag_type % 8)
            }
        }
        else {
            1 << self.tag_type
        }
    }
}

impl RealUdiBuf {
    fn new_raw(path: udi_buf_path_t) -> *mut udi_buf_t {
        Box::into_raw(Box::new(RealUdiBuf {
            raw: udi_buf_t { buf_size: 0 },
            inner: vec![],
            tags: vec![],
            path,
        })) as _
    }
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

    fn invalidate_tags(&mut self, off: udi_size_t, len: udi_size_t) {
        let end = off + len;
        self.tags.retain(|tag| {
            let tag_end = tag.tag_off+tag.tag_len;
            // Since `off<=off+len` - if the max of the start is before the min of the ends there is overlap
            let overlap = usize::max(off, tag.tag_off) < usize::min(end, tag_end);
            !overlap
        });
    }

    /// Delete data within a range
    fn delete_at(&mut self, off: udi_size_t, count: udi_size_t) {
        assert!(off <= self.len());
        assert!(count <= self.len());
        assert!(off+count <= self.len());
        self.invalidate_tags(off, count);
        self.inner.copy_within(off+count.., count);
        let new_len = self.len() - count;
        self.inner.truncate(new_len);
        self.raw.buf_size = new_len;

        // Update the tag offsets. None should overlap due to `invalidate_tags` above
        for tag in self.tags.iter_mut() {
            if tag.tag_off >= off {
                tag.tag_off -= count;
            }
        }
    }
    /// Insert zeros (undefined) at the given offset
    fn reserve_at(&mut self, off: udi_size_t, count: udi_size_t) {
        assert!(off <= self.len());
        self.invalidate_tags(off, 0);   // Zero length
        let old_len = self.len();
        self.inner.resize(count, 0);
        self.inner.copy_within(off..old_len, off+count);
        self.raw.buf_size = self.inner.len();

        // Update the tag offsets. None should overlap due to `invalidate_tags` above
        for tag in self.tags.iter_mut() {
            if tag.tag_off >= off {
                tag.tag_off -= count;
            }
        }
    }
}
unsafe fn get_or_alloc(ptr: &mut *mut udi_buf_t, path_handle: udi_buf_path_t) -> &mut RealUdiBuf {
    if ptr.is_null() {
        *ptr = RealUdiBuf::new_raw(path_handle);
    }
    &mut *(*ptr as *mut RealUdiBuf)
}
unsafe fn get_buf_mut(ptr: &mut *mut udi_buf_t) -> Option<&mut RealUdiBuf> {
    if ptr.is_null() {
        None
    }
    else {
        Some( &mut *(*ptr as *mut RealUdiBuf) )
    }
}
unsafe fn get_buf(ptr: &*mut udi_buf_t) -> Option<&RealUdiBuf> {
    if ptr.is_null() {
        None
    }
    else {
        Some( &*(*ptr as *mut RealUdiBuf) )
    }
}

/// Allocate a buffer for internal use
pub fn allocate(size: udi_size_t, path_handle: udi_buf_path_t) -> *mut udi_buf_t {
    let mut rv: *mut udi_buf_t = ::core::ptr::null_mut();
    // SAFE: It's null, so valid
    unsafe {
        get_or_alloc(&mut rv, path_handle).reserve_at(0, size);
    }
    rv
}
/// Read from a buffer
pub unsafe fn read(buf_ptr: *mut udi_buf_t, off: usize, dst: &mut [u8]) -> Option<usize> {
    if let Some(p) = get_buf(&buf_ptr) {
        if off < p.len() && off+dst.len() <= p.len() {
            let src = p.get_slice(off, dst.len());
            dst.copy_from_slice(src);
            Some(dst.len())
        }
        else {
            None
        }
    }
    else {
        None
    }
}
/// Write to a buffer, resizing using zeros if ranges don't match in size
pub unsafe fn write(buf_ptr: &mut *mut udi_buf_t, dst: ::core::ops::Range<usize>, src: &[u8]) {
    let p = get_or_alloc(buf_ptr, ::udi::ffi::buf::UDI_NULL_PATH_BUF);
    let dst_len = dst.end - dst.start;
    if dst_len < src.len() {
        p.reserve_at(dst.end, src.len() - dst_len);
    }
    else if src.len() < dst_len {
        p.delete_at(dst.start + src.len(), dst_len - src.len());
    }
    else {
        // No size change
    }
    p.invalidate_tags(dst.start, src.len());
    p.get_slice_mut(dst.start, src.len()).copy_from_slice(src);
}
pub unsafe fn get_mut(buf_ptr: &mut *mut udi_buf_t, range: ::core::ops::Range<usize>) -> &mut [u8] {
    let Some(p) = get_buf_mut(buf_ptr) else { panic!() };
    p.get_slice_mut(range.start, range.end - range.start)
}

/// [udi_buf_copy] logically replaces dst_len bytes of data starting at offset
/// dst_offset in dst_buf with a copy of src_len bytes of data starting at
/// src_offset in src_buf. When the data has been copied, the callback
/// routine is called.
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
    path_handle: udi_buf_path_t
)
{
    assert!(src_buf != dst_buf, "Not allowed to reference the same buffer");
    let src = get_buf(&src_buf).unwrap().get_slice(src_off, src_len);
    // TODO: Also need to copy the tags
    udi_buf_write(callback, gcb, src.as_ptr() as *const c_void, src.len(), dst_buf, dst_off, dst_len, path_handle);
}
#[no_mangle]
unsafe extern "C" fn udi_buf_write(
    callback: udi_buf_copy_call_t,
    gcb: *mut udi_cb_t,
    src_buf: *const c_void,
    src_len: udi_size_t,
    mut dst_buf: *mut udi_buf_t,
    dst_off: udi_size_t,
    dst_len: udi_size_t,
    path_handle: udi_buf_path_t
)
{
    let dst = get_or_alloc(&mut dst_buf, path_handle);
    dst.invalidate_tags(dst_off, dst_len);
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
unsafe extern "C" fn udi_buf_read(
    src_buf: *mut udi_buf_t,
    src_off: udi_size_t,
    src_len: udi_size_t,
    dst_mem: *mut c_void,
)
{
    let src = get_buf(&src_buf).unwrap().get_slice(src_off, src_len);
    ::core::ptr::copy_nonoverlapping(src.as_ptr(), dst_mem as *mut u8, src.len());
}


#[no_mangle]
unsafe extern "C" fn udi_buf_free(buf: *mut udi_buf_t)
{
    if buf.is_null() {
    }
    else {
        drop(Box::from_raw(buf as *mut RealUdiBuf));
    }
}

#[no_mangle]
unsafe extern "C" fn udi_buf_best_path(
    buf: *mut udi_buf_t,
    path_handles: *mut udi_buf_path_t,
    npaths: ::udi::ffi::udi_ubit8_t,
    last_fit: ::udi::ffi::udi_ubit8_t,
    best_fit_array: *mut ::udi::ffi::udi_ubit8_t
)
{
    let path_handles = ::core::slice::from_raw_parts_mut(path_handles, npaths as usize);
    let best_fit_array = ::core::slice::from_raw_parts_mut(best_fit_array, 1 + npaths as usize);
    if let Some(buf) = get_buf(&buf)
    {
        let _ = path_handles;
        let _ = buf;
        // HACK: Just consider all handles to be equally good
        // - A better option would be to look at the constraint list in the path and see if any match this buffer's layout
        // - Except that DMA always bounces in this implementation, so meh.
        let mut dst_it = best_fit_array.iter_mut();
        for (dst, (i,_)) in Iterator::zip(dst_it.by_ref(), path_handles.iter().enumerate()) {
            *dst = ((i + last_fit as usize + 1) % path_handles.len()) as u8;
        }
        *dst_it.next().unwrap() = ::udi::ffi::buf::UDI_BUF_PATH_END;
    }
}
#[no_mangle]
unsafe extern "C" fn udi_buf_path_alloc(
    callback: ::udi::ffi::buf::udi_buf_path_alloc_call_t,
    gcb: *mut udi_cb_t,
)
{
    let rv = Box::into_raw(Box::new(RealPath {})) as udi_buf_path_t;
    callback(gcb, rv);
}
#[no_mangle]
unsafe extern "C" fn udi_buf_path_free(buf_path: udi_buf_path_t)
{
    drop(Box::from_raw(buf_path as *mut RealPath));
}


#[no_mangle]
unsafe extern "C" fn udi_buf_tag_set(
    callback: ::udi::ffi::buf::udi_buf_tag_set_call_t,
    gcb: *mut udi_cb_t,
    mut buf: *mut udi_buf_t,
    tag_array: *mut udi_buf_tag_t,
    tag_array_length: ::udi::ffi::udi_ubit16_t,
)
{
    let tags = ::core::slice::from_raw_parts(tag_array as *const udi_buf_tag_t, tag_array_length as usize);

    if let Some(p) = get_buf_mut(&mut buf)
    {
        for tag in tags.iter()
        {
            let tag = Tag {
                tag_off: tag.tag_off,
                tag_len: tag.tag_len,
                tag_type: {
                    let idx = tag.tag_type.trailing_zeros();
                    if idx >= 24 {
                        let driver_index: u32 = todo!("per-driver buffer tags");
                        idx + driver_index * 8
                    }
                    else {
                        idx
                    }
                    },
                tag_value: tag.tag_value,
            };
            match p.tags.binary_search_by(|v: &Tag| tag.tag_off.cmp(&v.tag_off).then(tag.tag_len.cmp(&v.tag_len)).then(tag.tag_type.cmp(&v.tag_type)))
            {
            Ok(pos) => {
                if tag.tag_type < 24 {
                    p.tags[pos].tag_value = tag.tag_value;
                }
                else {
                    p.tags.insert(pos, tag);
                }
                },
            Err(pos) => {
                p.tags.insert(pos, tag);
                },
            }
        }
    }
    callback(gcb, buf);
}

#[no_mangle]
unsafe extern "C" fn udi_buf_tag_get(
    buf: *mut udi_buf_t,
    tag_type: ::udi::ffi::buf::udi_tagtype_t,
    tag_array: *mut ::udi::ffi::buf::udi_buf_tag_t,
    tag_array_length: ::udi::ffi::udi_ubit16_t,
    mut tag_start_idx: ::udi::ffi::udi_ubit16_t,
) -> ::udi::ffi::udi_ubit16_t
{
    let tags = ::core::slice::from_raw_parts_mut(tag_array, tag_array_length as usize);
    let Some(buf) = get_buf(&buf) else { return 0; };

    let mut rv = 0;
    for tag in buf.tags.iter()
    {
        if tag.get_type(0) & tag_type != 0
        {
            if tag_start_idx > 0 {
                tag_start_idx -= 1;
            }
            else {
                tags[rv] = udi_buf_tag_t {
                    tag_off: tag.tag_off,
                    tag_len: tag.tag_len,
                    tag_type: tag.get_type(0),
                    tag_value: tag.tag_value,
                };
                rv += 1;
                if rv == tags.len() {
                    break;
                }
            }
        }
    }

    rv as u16
}

#[no_mangle]
unsafe extern "C" fn udi_buf_tag_compute(
    buf: *mut udi_buf_t,
    off: udi_size_t,
    len: udi_size_t,
    tag_type: ::udi::ffi::buf::udi_tagtype_t,
) -> ::udi::ffi::udi_ubit32_t
{
    let Some(buf) = get_buf(&buf) else { return 0 };
    let val = buf.get_slice(off, len);
    match tag_type & ::udi::ffi::buf::UDI_BUFTAG_VALUES
    {
    ::udi::ffi::buf::UDI_BUFTAG_BE16_CHECKSUM => {
        let mut rv = 0;
        for pair in val.chunks(2) {
            if pair.len() == 2 {
                rv += u16::from_be_bytes([pair[0], pair[1]]);
            }
            else {
                rv += u16::from_be_bytes([pair[0], 0]);
            }
        }
        rv as u32
    }
    v => todo!("udi_buf_tag_compute: {:#x}", v),
    }
}

#[no_mangle]
unsafe extern "C" fn udi_buf_tag_apply(
    callback: ::udi::ffi::buf::udi_buf_tag_set_apply_t,
    gcb: *mut udi_cb_t,
    mut buf: *mut udi_buf_t,
    tag_type: ::udi::ffi::buf::udi_tagtype_t,
)
{
    if let Some(p) = get_buf_mut(&mut buf)
    {
        for tags in p.tags.iter()
        {
            if tags.tag_type & tag_type & ::udi::ffi::buf::UDI_BUFTAG_UPDATES != 0
            {
                match tags.tag_type
                {
                _ => {},
                }
            }
            todo!("udi_buf_tag_apply");
        }
    }
    callback(gcb, buf);
}