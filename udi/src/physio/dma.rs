use ::core::future::Future;
use ::udi_sys::physio as ffi;
use ::udi_sys::physio::udi_dma_constraints_attr_spec_t;
use ::udi_sys::physio::udi_dma_constraints_t;
use ::udi_sys::physio::udi_dma_handle_t;

#[derive(Debug)]
pub struct DmaConstraints(udi_dma_constraints_t);
impl Drop for DmaConstraints
{
    fn drop(&mut self) {
        unsafe {
            ::udi_sys::physio::udi_dma_constraints_free(self.0)
        }
    }
}
impl Default for DmaConstraints {
    fn default() -> Self {
        Self::null()
    }
}
impl DmaConstraints
{
    pub fn null() -> DmaConstraints {
        DmaConstraints(::udi_sys::physio::UDI_NULL_DMA_CONSTRAINTS)
    }
    pub unsafe fn from_raw(v: udi_dma_constraints_t) -> Self {
        DmaConstraints(v)
    }

    /// Reset the specifided attribute to its default (usually the least restrictive)
    pub fn reset(&mut self, attr_type: ::udi_sys::physio::udi_dma_constraints_attr_t)
    {
        unsafe {
            ::udi_sys::physio::udi_dma_constraints_attr_reset(self.0, attr_type)
        }
    }

    /// Set a collection of attributes
    pub fn set<'a>(
        &'a mut self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        attrs: &'a [udi_dma_constraints_attr_spec_t]
    ) -> impl Future<Output=crate::Result<()>> + 'a
    {
        unsafe extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t, new_ptr: udi_dma_constraints_t, status: ::udi_sys::udi_status_t) {
            let res = crate::Error::from_status(status).map(|()| new_ptr as _);
            crate::async_trickery::signal_waiter(&mut *gcb, crate::async_trickery::WaitRes::PointerResult(res))
        }
        let src_constraints = self.0;
        crate::async_trickery::wait_task(
            gcb,
            move |gcb| unsafe {
                ::udi_sys::physio::udi_dma_constraints_attr_set(
                    callback, gcb, src_constraints, attrs.as_ptr(), attrs.len() as _, 0
                )
            },
            move |res|
                match res {
                crate::async_trickery::WaitRes::PointerResult(v) => match v
                    {
                    Ok(p) => { self.0 = p as _; Ok(())},
                    Err(e) => Err(e),
                    },
                _ => panic!(),
                }
            )
    }
}

pub enum Direction {
    In,
    Out,
    Both,
}
impl Direction {
    fn to_flags(&self) -> u8 {
        match self {
        Direction::In => ffi::UDI_DMA_IN,
        Direction::Out => ffi::UDI_DMA_OUT,
        Direction::Both => ffi::UDI_DMA_IN|ffi::UDI_DMA_OUT,
        }
    }
}
pub enum Endianness {
    Big,
    Little,
    NeverSwap,
}
impl Endianness {
    fn to_flags(&self) -> u8 {
        match self {
        Endianness::Big => ffi::UDI_DMA_BIG_ENDIAN,
        Endianness::Little => ffi::UDI_DMA_LITTLE_ENDIAN,
        Endianness::NeverSwap => ffi::UDI_DMA_NEVERSWAP,
        }
    }
}

fn range_to_ofs_len(max_ofs: usize, range: impl ::core::ops::RangeBounds<usize>) -> (usize, usize) {
    let offset = match range.start_bound() {
        core::ops::Bound::Included(&v) => v,
        core::ops::Bound::Excluded(&v) => v+1,
        core::ops::Bound::Unbounded => 0,
        };
    let end = match range.end_bound() {
        core::ops::Bound::Included(&v) => v+1,
        core::ops::Bound::Excluded(&v) => v,
        core::ops::Bound::Unbounded => max_ofs,
        };
    let len = end - offset;
    (offset, len)
}

// Internal DMA handle for common operations
struct DmaHandle(udi_dma_handle_t);
impl Drop for DmaHandle {
    fn drop(&mut self) {
        unsafe {
            ffi::udi_dma_free(self.0);
        }
    }
}
impl DmaHandle {
    /// Synchronise driver/device views of all of the memory
    pub fn sync<'a>(
        &'a self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        offset: usize,
        length: usize,
        dir: Direction,
    ) -> impl Future<Output=()> + 'a {
        unsafe extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t) {
            crate::async_trickery::signal_waiter(&mut *gcb, crate::async_trickery::WaitRes::Pointer(0 as _))
        }
        crate::async_trickery::wait_task(gcb,
            move |gcb| unsafe {
                ::udi_sys::physio::udi_dma_sync(callback, gcb, self.0, offset, length, dir.to_flags());
            },
            |_res| (),
        )
    }
    /// **Only need if the device/driver has written to the scatter-gather list directly**
    /// 
    /// Synchronise between driver and device views of the scatter-gather list
    pub fn scgth_sync<'a>(&'a self, gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>) -> impl Future<Output=()> + 'a {
        unsafe extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t) {
            crate::async_trickery::signal_waiter(&mut *gcb, crate::async_trickery::WaitRes::Pointer(0 as _))
        }
        crate::async_trickery::wait_task(gcb,
            move |gcb| unsafe {
                ::udi_sys::physio::udi_dma_scgth_sync(callback, gcb, self.0);
            },
            |_res| (),
        )
    }
    /// Request a CPU memory barrier for all memory associated with the DMA handle
    pub fn mem_barrier(&self) {
        unsafe { ffi::udi_dma_mem_barrier(self.0) }
    }
}

/// Handle to allocated DMA memory for use with buffers
pub struct DmaBuf {
    handle: DmaHandle,
    cur_buf: Option<(*mut ::udi_sys::udi_buf_t, usize, usize, u8)>
}
impl Drop for DmaBuf {
    fn drop(&mut self) {
        unsafe {
            // If there's a buffer currently owned, then free it
            if let Some((buf, ..)) = self.cur_buf {
                let _ = crate::buf::Handle::from_raw(buf);
            }
        }
    }
}
impl DmaBuf {
    /// Prepare a DMA handle for buffer usage
    pub fn prepare<'a>(gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>, constraints: &'a DmaConstraints, dir_hint: Option<Direction>) -> impl Future<Output=Self> + 'a {
        let flags = match dir_hint {
            None => 0,
            Some(d) => d.to_flags(),
        };
        unsafe extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t, new_ptr: udi_dma_handle_t) {
            crate::async_trickery::signal_waiter(&mut *gcb, crate::async_trickery::WaitRes::Pointer(new_ptr as _))
        }
        crate::async_trickery::wait_task(gcb,
            move |gcb| unsafe { ::udi_sys::physio::udi_dma_prepare(callback, gcb, constraints.0, flags) },
            |res| {
                let crate::async_trickery::WaitRes::Pointer(p) = res else { panic!() };
                Self { 
                    handle: DmaHandle(p as _),
                    cur_buf: None
                }
            }
        )
    }

    /// Map a buffer for DMA
    /// - Takes ownership of `buf`, to be released by `buf_unmap`
    /// 
    /// If the returned `bool` is `false`, the ScGth structure only contains part of the input buffer.
    /// When this happens, the driver should call [Self::buf_map_continue] until it returns `true`
    pub fn buf_map<'a>(
        &'a mut self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        buf: crate::buf::Handle,
        range: impl ::core::ops::RangeBounds<usize>,
        dir: Direction,
    ) -> impl Future<Output=crate::Result<(ScGth,bool)>> + 'a {
        let (offset, len) = range_to_ofs_len(buf.len(), range);
        let flags = 0
            | dir.to_flags()
            ;
        let buf = buf.into_raw();
        if self.cur_buf.is_some() {
            // TODO: Release this buffer? panic (ordering error)?
        }
        self.cur_buf = Some( (buf, offset, len, flags) );

        // Note: Always passes `ffi::UDI_DMA_REWIND` to ensure that even if the input happens to be identical it behave the same
        // SAFE: Correct arguments
        unsafe { self.buf_map_inner(gcb, buf, offset, len, flags | ffi::UDI_DMA_REWIND) }
    }
    /// Continue iterating a previous call to [Self::buf_map]
    /// 
    /// `rewind` requrests that the iteration restart from the start of the buffer range
    pub fn buf_map_continue<'a>(
        &'a mut self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        rewind: bool
    ) -> impl Future<Output=crate::Result<(ScGth,bool)>> + 'a {
        let Some( (buf, offset, len, flags) ) = self.cur_buf else {
            panic!("Incorrect call to `buf_map_continue` without a previous call to `buf_map`");
        };
        let flags = flags | if rewind { ffi::UDI_DMA_REWIND } else { 0 };
        // SAFE: Correct arguments
        unsafe { self.buf_map_inner(gcb, buf, offset, len, flags) }
    }

    /// SAFETY: The caller must ensure that `buf` and `flags` are valid
    unsafe fn buf_map_inner<'a>(
        &'a mut self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        buf: *mut ::udi_sys::udi_buf_t,
        offset: usize, len: usize,
        flags: u8
    ) -> impl Future<Output=crate::Result<(ScGth,bool)>> + 'a {
        unsafe extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t, scgth: *mut ffi::udi_scgth_t, complete: ::udi_sys::udi_boolean_t, status: ::udi_sys::udi_status_t) {
            let res = crate::async_trickery::WaitRes::Data([
                status as _,
                scgth as _,
                complete.0 as _,
                0,
            ]);
            crate::async_trickery::signal_waiter(&mut *gcb, res)
        }
        crate::async_trickery::wait_task(gcb,
            move |gcb| unsafe { ::udi_sys::physio::udi_dma_buf_map(callback, gcb, self.handle.0, buf, offset, len, flags) },
            |res| {
                let crate::async_trickery::WaitRes::Data([status, scgth, is_complete, ..]) = res else { panic!() };
                crate::Error::from_status(status as _)
                    .map(|()| (ScGth::from_raw(scgth as _), is_complete != 0))
                },
        )
    }

    /// SAFETY: The device must no longer be accessing the buffer using this handle
    pub unsafe fn buf_unmap(&mut self, new_buf_size: usize) -> Option<crate::buf::Handle> {
        // TODO: How to ensure that there is a mapped buffer to return
        if let Some((_, ..)) = self.cur_buf.take() {
            unsafe {
                Some( crate::buf::Handle::from_raw( ffi::udi_dma_buf_unmap(self.handle.0, new_buf_size) ) )
            }
        }
        else {
            None
        }
    }

    /// Synchronise driver/device views of all of the memory
    pub fn sync_all<'a>(
        &'a self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        dir: Direction,
    ) -> impl Future<Output=()> + 'a {
        self.handle.sync(gcb, 0, 0, dir)
    }
    /// Synchronise driver/device views of part of the memory
    pub fn sync<'a>(
        &'a self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        range: impl ::core::ops::RangeBounds<usize>,
        dir: Direction,
    ) -> impl Future<Output=()> + 'a {
        let Some((_,_,input_length,_)) = self.cur_buf else { panic!("Incorrect call to `sync` without a previous call to `buf_map`") };
        let (offset, length) = range_to_ofs_len(input_length, range);
        self.handle.sync(gcb, offset, length, dir)
    }
    /// **Only need if the device/driver has written to the scatter-gather list directly**
    /// 
    /// Synchronise between driver and device views of the scatter-gather list
    pub fn scgth_sync<'a>(&'a self, gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>) -> impl Future<Output=()> + 'a {
        self.handle.scgth_sync(gcb)
    }
    /// Request a CPU memory barrier for all memory associated with the DMA handle
    pub fn mem_barrier(&self) {
        self.handle.mem_barrier();
    }
}

pub struct DmaAlloc {
    /// DMA allocation handle
    handle: DmaHandle,
    /// Scatter-gather entries
    scgth: ScGth<'static>,  // Thie `'static` is a lie, it's actually `'self`
    /// Driver-mapped pointer to the allocated DMA-able memory.
    pub mem_ptr: *mut ::udi_sys::c_void,
    /// If `gap_size` is None, then only a single element was allocated (not an error if doing a single element alloc)
    pub gap_size: Option<usize>,
    /// Indicates that the environment has determined that the device/system/driver endian don't match, and the driver must swap the
    /// endianess of the values in this allocation
    pub must_swap: bool,
}
impl Drop for DmaAlloc {
    fn drop(&mut self) {
    }
}
impl DmaAlloc {
    /// Allocate DMA-able memory for shared device structures
    /// 
    /// Errors:
    /// - `UDI_STAT_RESOURCE_UNAVAIL` if the mapping would have been partial and `UDI_DMA_NO_PARTIAL` was specified
    pub fn alloc<'a>(
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        constraints: &'a DmaConstraints,
        dir: Direction,
        endian: Endianness,
        nozero: bool,
        nelements: u16,
        element_size: usize,
        max_gap: usize
    ) -> impl Future<Output=Self> + 'a
    {
        let flags = 0
            | dir.to_flags()
            | endian.to_flags()
            | if nozero { ::udi_sys::mem::UDI_MEM_NOZERO } else { 0 }
            ;
            unsafe extern "C" fn callback(
                gcb: *mut ::udi_sys::udi_cb_t,
                new_ptr: udi_dma_handle_t,
                mem_ptr: *mut ::udi_sys::c_void,
                actual_gap: ::udi_sys::udi_size_t,
                single_element: ::udi_sys::udi_boolean_t,
                scgth: *mut ffi::udi_scgth_t,
                must_swap: ::udi_sys::udi_boolean_t,
            ) {
                let actual_gap = if single_element.to_bool() { 0 } else { actual_gap };
                // Make sure that we can hackily fit two flag bits in here
                assert!(actual_gap <= usize::MAX >> 2);
                let res = crate::async_trickery::WaitRes::Data([
                    new_ptr as usize,
                    mem_ptr as usize,
                    scgth as usize,
                    (actual_gap as usize) << 2
                    | if single_element.to_bool() { 1 << 0 } else { 0 }
                    | if must_swap.to_bool() { 1 << 1 } else { 0 }
                ]);
                crate::async_trickery::signal_waiter(&mut *gcb, res)
            }
            unsafe extern "C" fn callback_single(
                gcb: *mut ::udi_sys::udi_cb_t,
                new_ptr: udi_dma_handle_t,
                mem_ptr: *mut ::udi_sys::c_void,
                _actual_gap: ::udi_sys::udi_size_t,
                _single_element: ::udi_sys::udi_boolean_t,
                scgth: *mut ffi::udi_scgth_t,
                must_swap: ::udi_sys::udi_boolean_t,
            ) {
                let res = crate::async_trickery::WaitRes::Data([
                    new_ptr as usize,
                    mem_ptr as usize,
                    scgth as usize,
                    0
                    | if must_swap.to_bool() { 1 << 1 } else { 0 }
                ]);
                crate::async_trickery::signal_waiter(&mut *gcb, res);
            }
            // Use a simpler callback when allocating a single element
            let callback = if nelements == 1 { callback_single } else { callback };
            crate::async_trickery::wait_task(gcb,
                move |gcb| unsafe {
                    ::udi_sys::physio::udi_dma_mem_alloc(callback, gcb, constraints.0, flags, nelements, element_size, max_gap)
                },
                |res| {
                    let crate::async_trickery::WaitRes::Data([new_ptr, mem_ptr, scgth, gap_flags]) = res else { panic!() };
                    let single_element = gap_flags & 1 != 0;
                    let must_swap = gap_flags & 2 != 0;
                    let gap_size = gap_flags >> 2;
                    DmaAlloc {
                        handle: DmaHandle(new_ptr as _),
                        scgth: unsafe { ScGth::from_raw(scgth as _) },
                        mem_ptr: mem_ptr as _,
                        gap_size: if single_element { None } else { Some(gap_size) },
                        must_swap }
                    },
            )
    }

    /// Simpler version of [DmaAlloc::alloc] that sets `nelements=1`
    pub fn alloc_single<'a>(
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        constraints: &'a DmaConstraints,
        dir: Direction,
        endian: Endianness,
        nozero: bool,
        element_size: usize,
    ) -> impl Future<Output=Self> + 'a
    {
        Self::alloc(gcb, constraints, dir, endian, nozero, 1, element_size, 0)
    }

    /// Get access to the scatter-gather list
    pub fn scgth(&self) -> &ScGth {
        &self.scgth
    }

    /// Synchronise driver/device views of all of the memory
    pub fn sync_all<'a>(
        &'a self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        dir: Direction,
    ) -> impl Future<Output=()> + 'a {
        self.handle.sync(gcb, 0, 0, dir)
    }
    /// **Only need if the device/driver has written to the scatter-gather list directly**
    /// 
    /// Synchronise between driver and device views of the scatter-gather list
    pub fn scgth_sync<'a>(&'a self, gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>) -> impl Future<Output=()> + 'a {
        self.handle.scgth_sync(gcb)
    }
    /// Request a CPU memory barrier for all memory associated with the DMA handle
    pub fn mem_barrier(&self) {
        self.handle.mem_barrier();
    }

    // TODO: `mem_to_buf`
    #[cfg(false_)]
    /// Free the DMA handle, and copy the data within `src_range` to the output buffer (as the full buffer contents)
    pub fn mem_to_buf(
        self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        src_range: impl ::core::ops::RangeBounds<usize>,
        dst_buf: crate::buf::Handle
    ) -> Future<Output=crate::buf::Handle>
    {
        let (offset, len) = range_to_ofs_len(self.len, range);
    }
}

// Note: I'm assuming that it's a borrow from the DMA handle
pub struct ScGth<'a>(&'a ffi::udi_scgth_t);
impl<'a> ScGth<'a> {
    unsafe fn from_raw(p: *const ffi::udi_scgth_t) -> Self {
        ScGth(&*p)
    }
    pub fn raw_entries(&self) -> ScgthRaw {
        let len = self.0.scgth_num_elements as _;
        unsafe {
            match self.0.scgth_format {
            ffi::UDI_SCGTH_32 => ScgthRaw::Bits32( ::core::slice::from_raw_parts(self.0.scgth_elements.el32p, len) ),
            ffi::UDI_SCGTH_64 => ScgthRaw::Bits64( ::core::slice::from_raw_parts(self.0.scgth_elements.el64p, len) ),
            _ => panic!("Malformed `scgth_format` {} ({:p})", self.0.scgth_format, self.0),
            }
        }
    }
    pub fn single_entry_32(&self) -> Option<&ffi::udi_scgth_element_32_t> {
        match self.raw_entries() {
        ScgthRaw::Bits32(&[ref ent]) => Some(ent),
        ScgthRaw::Bits32(_) => None,
        ScgthRaw::Bits64(_) => None,
        }
    }
}
/// Raw IEEE 1212.1 Scatter-gather elements
pub enum ScgthRaw<'a> {
    Bits32(&'a [ffi::udi_scgth_element_32_t]),
    Bits64(&'a [ffi::udi_scgth_element_64_t]),
}