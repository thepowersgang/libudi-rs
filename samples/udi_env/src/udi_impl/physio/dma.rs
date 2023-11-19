use ::udi::ffi::physio as ffi;
use ::udi::ffi::{udi_ubit8_t,udi_ubit16_t,udi_size_t};
use ::udi::ffi::{udi_cb_t,udi_buf_t};

/// Inner data for `udi_dma_handle_t`
struct DmaHandleInner {
    backing: BackingData,
    dma_info: Option<DmaInfo>,
}

/// Backing type (buffer or raw allocation)
enum BackingData {
    Buffer(BackingBuffer),
    RawData(BackingRaw),
}
struct BackingBuffer {
    /// Constraints passed to `udi_dma_prepare`
    constraints: super::dma_constraints::ConstaintsReal,
    /// Mapped buffer
    buf: *mut udi_buf_t,
    /// Offset+Length from the most recent `map`` call
    range: ::core::ops::Range<usize>,
    /// Direction from the most recent `map` call
    dir: Direction,
}
struct BackingRaw {
    #[allow(dead_code)]
    dir: Direction,
    /// Size
    size: usize,
    /// The raw buffer
    buffer: *mut ::core::ffi::c_void,
}
struct DmaInfo {
    data_handle: crate::emulated_devices::DmaHandle,
    scgth_handle: crate::emulated_devices::DmaHandle,
    scgth: ffi::udi_scgth_t,
}
impl DmaInfo {
    fn alloc(device: &dyn crate::emulated_devices::PioDevice, _constraints: &super::dma_constraints::ConstaintsReal, data_size: usize) -> DmaInfo {
        let data_handle = device.dma_alloc(data_size);
        let scgth_data = vec![
            ffi::udi_scgth_element_32_t { block_busaddr: data_handle.addr(), block_length: data_handle.len(), },
        ].into_boxed_slice();
        DmaInfo {
            data_handle,
            scgth_handle: device.dma_alloc(::core::mem::size_of_val(&*scgth_data)),
            scgth: ffi::udi_scgth_t {
                scgth_num_elements: scgth_data.len() as _,
                scgth_format: ffi::UDI_SCGTH_32,
                scgth_must_swap: ::udi::ffi::FALSE,
                scgth_first_segment: ffi::udi_scgth_t_scgth_first_segment {
                    el32: scgth_data[0],
                },
                scgth_elements: ffi::udi_scgth_t_scgth_elements { el32p: Box::into_raw(scgth_data) as *mut _ },
            }
        }
    }
}

#[derive(PartialEq)]
enum Direction {
    In,
    Out,
    BiDir,
}
impl Direction {
    fn from_flags(flags: u8) -> Option<Direction> {
        match flags & (ffi::UDI_DMA_IN|ffi::UDI_DMA_OUT) {
        0 => None,
        ffi::UDI_DMA_IN => Some(Direction::In),
        ffi::UDI_DMA_OUT => Some(Direction::Out),
        _ => Some(Direction::BiDir),
        }
    }
}

#[no_mangle]
unsafe extern "C" fn udi_dma_prepare(
    callback: ffi::udi_dma_prepare_call_t,
    gcb: *mut udi_cb_t,
    constraints: ffi::udi_dma_constraints_t,
    flags: udi_ubit8_t
)
{
    let instance = crate::channels::get_driver_instance( &(*gcb).channel );
    let _device = &**instance.device.get().expect("Calling `udi_dma_prepare` with no device");
    let constraints = super::dma_constraints::ConstaintsReal::from_ref(&constraints);
    let _dir = Direction::from_flags(flags);

    let rv = Box::new(DmaHandleInner {
        backing: BackingData::Buffer(BackingBuffer {
            constraints: constraints.clone(),
            buf: ::core::ptr::null_mut(),
            range: 0..0, dir: Direction::BiDir
        }),
        dma_info: None,
    });

    callback(gcb, Box::into_raw(rv) as ffi::udi_dma_handle_t)
}

#[no_mangle]
unsafe extern "C" fn udi_dma_buf_map(
    callback: ffi::udi_dma_buf_map_call_t,
    gcb: *mut udi_cb_t,
    dma_handle: ffi::udi_dma_handle_t,
    buf: *mut udi_buf_t,
    offset: udi_size_t,
    len: udi_size_t,
    flags: udi_ubit8_t
)
{
    let dma_handle = &mut *(dma_handle as *mut DmaHandleInner);
    let instance = crate::channels::get_driver_instance( &(*gcb).channel );
    let device = &**instance.device.get().unwrap();

    let Some(dir) = Direction::from_flags(flags) else {
        panic!("`udi_dma_buf_map` with no direction flags")
    };
    let is_rewind = flags & ffi::UDI_DMA_REWIND != 0;
    let range = offset..(offset+len);

    let BackingData::Buffer(ref mut backing) = dma_handle.backing else {
        panic!("`udi_dma_buf_map` on a pre-allocated buffer");
    };

    // Check if the buffer/offset/len/dir is the same
    if !is_rewind && backing.buf == buf && backing.range == range && backing.dir == dir {
        // Same, so resume
        todo!("Resume a previous run")
    }
    else {
        if !backing.buf.is_null() {
            todo!("udi_dma_buf_map: What happens when the buffer is already set")
        }
        // Rewind (or new)
        backing.buf = buf;
        backing.range = range.clone();
        backing.dir = dir;
    }

    dma_handle.dma_info = Some(DmaInfo::alloc(device, &backing.constraints, range.end - range.start));

    let scgth = &mut dma_handle.dma_info.as_mut().unwrap().scgth as *mut _;
    let complete = ::udi::ffi::TRUE;
    callback(gcb, scgth, complete, ::udi::ffi::UDI_OK as _);
}

#[no_mangle]
unsafe extern "C" fn udi_dma_buf_unmap(
    dma_handle: ffi::udi_dma_handle_t,
    new_buf_size: udi_size_t
) -> *mut udi_buf_t
{
    let dma_handle = &mut *(dma_handle as *mut DmaHandleInner);
    let BackingData::Buffer(ref mut backing) = dma_handle.backing else {
        panic!("udi_dma_buf_unmap on non-buf DMA handle")
        };
    if !backing.buf.is_null() {
        assert!(new_buf_size <= (*backing.buf).buf_size);
        (*backing.buf).buf_size = new_buf_size;

        // Free DMA handles?
    }
    else {
        println!("Warning: `udi_dma_buf_unmap` called with no mapped buffer");
    }
    ::core::mem::replace(&mut backing.buf, ::core::ptr::null_mut())
}

#[no_mangle]
unsafe extern "C" fn udi_dma_mem_alloc(
    callback: ffi::udi_dma_mem_alloc_call_t,
    gcb: *mut udi_cb_t,
    constraints: ffi::udi_dma_constraints_t,
    flags: udi_ubit8_t,
    nelements: udi_ubit16_t,
    element_size: udi_size_t,
    max_gap: udi_size_t
)
{
    let instance = crate::channels::get_driver_instance( &(*gcb).channel );
    let dev = instance.device.get().expect("Calling `udi_dma_mem_alloc` with no registered device");
    let constraints = super::dma_constraints::ConstaintsReal::from_ref(&constraints);
    let Some(dir) = Direction::from_flags(flags) else {
        panic!("Calling `udi_dma_mem_alloc` with no direction flag set")
    };
    
    //let pad = element_size.next_multiple_of(64) - element_size;
    let pad = 0;
    if pad > max_gap {
        // Failure, return `single_element`
        todo!("udi_dma_mem_alloc: Handle returning single_element")
    }
    let total_size = (element_size + pad) * nelements as usize;
    let dma_info = DmaInfo::alloc(&**dev, constraints, total_size);

    let mem_ptr = ::libc::malloc(total_size);

    let mut rv = Box::new(DmaHandleInner {
        backing: BackingData::RawData(BackingRaw {
            dir,
            size: total_size,
            buffer: mem_ptr
        }),
        dma_info: Some(dma_info),
    });
    let scgth = &mut rv.dma_info.as_mut().unwrap().scgth as *mut _;
    let rv = Box::into_raw(rv) as ffi::udi_dma_handle_t;

    callback(gcb, rv, mem_ptr, pad, ::udi::ffi::TRUE, scgth, ::udi::ffi::FALSE);
}

#[no_mangle]
unsafe extern "C" fn udi_dma_sync(
    callback: ffi::udi_dma_sync_call_t,
    gcb: *mut udi_cb_t,
    dma_handle: ffi::udi_dma_handle_t,
    offset: udi_size_t,
    length: udi_size_t,
    flags: udi_ubit8_t
)
{
    let dma_handle = &mut *(dma_handle as *mut DmaHandleInner);
    let Some(dir) = Direction::from_flags(flags) else {
        panic!("Calling `udi_dma_sync` with no direction flags");
    };

    let dma_info = dma_handle.dma_info.as_mut().expect("Calling `udi_dma_sync` with no DMA mapping");

    let raw_data = match dma_handle.backing {
        BackingData::Buffer(ref mut backing) => {
            assert!(!backing.buf.is_null());
            todo!("udi_dma_sync with buffer")
        },
        BackingData::RawData(ref mut backing) => {
            match (&dir, &backing.dir)
            {
            // BiDir configured allows a request of anything
            (_, Direction::BiDir) => {},
            // - Matching requests
            (Direction::In, Direction::In) => {},
            (Direction::Out, Direction::Out) => {},
            //(Direction::BiDir, Direction::BiDir) => {},

            (Direction::In, Direction::Out)
            |(Direction::Out, Direction::In)
            |(Direction::BiDir, Direction::In)
            |(Direction::BiDir, Direction::Out) =>
                panic!("`udi_dma_sync` with non-matching directions"),
            }
            ::core::slice::from_raw_parts_mut(backing.buffer as *mut u8, backing.size)
        },
    };
    let raw_data = &mut raw_data[offset..][..length];
    match dir {
    Direction::In => dma_info.data_handle.read(offset, raw_data),
    Direction::Out => dma_info.data_handle.write(offset, raw_data),
    Direction::BiDir => todo!("udi_dma_sync In+Out"),
    }

    callback(gcb);
}

#[no_mangle]
unsafe extern "C" fn udi_dma_scgth_sync(
    callback: ffi::udi_dma_scgth_sync_call_t,
    gcb: *mut udi_cb_t,
    dma_handle: ffi::udi_dma_handle_t
)
{
    let dma_handle = &mut *(dma_handle as *mut DmaHandleInner);
    let dma_info = dma_handle.dma_info.as_mut().expect("Calling `udi_dma_scgth_sync` with no DMA mapping");
    let dst = {
        let ele_size = if dma_info.scgth.scgth_format == ffi::UDI_SCGTH_32 {
            ::core::mem::size_of::<ffi::udi_scgth_element_32_t>()
        }
        else {
            ::core::mem::size_of::<ffi::udi_scgth_element_64_t>()
        };
        ::core::slice::from_raw_parts_mut(dma_info.scgth.scgth_elements.el32p as *mut u8, ele_size * dma_info.scgth.scgth_num_elements as usize)
    };
    dma_info.scgth_handle.read(0, dst);
    callback(gcb);
}


#[no_mangle]
unsafe extern "C" fn udi_dma_mem_barrier(dma_handle: ffi::udi_dma_handle_t)
{
    let _dma_handle = &mut *(dma_handle as *mut DmaHandleInner);
}

#[no_mangle]
unsafe extern "C" fn udi_dma_free(dma_handle: ffi::udi_dma_handle_t)
{
    drop(Box::from_raw(dma_handle as *mut DmaHandleInner));
}

#[no_mangle]
unsafe extern "C" fn udi_dma_mem_to_buf(
    callback: ffi::udi_dma_mem_to_buf_call_t,
    gcb: *mut udi_cb_t,
    dma_handle: ffi::udi_dma_handle_t,
    src_off: udi_size_t,
    src_len: udi_size_t,
    mut dst_buf: *mut udi_buf_t
)
{
    let dma_handle = &mut *(dma_handle as *mut DmaHandleInner);
    let backing = match dma_handle.backing {
        BackingData::Buffer(_) => panic!("`udi_dma_mem_to_buf` with buffer-mapped handle"),
        BackingData::RawData(ref mut backing) => backing,
        };
    
    let data = ::core::slice::from_raw_parts(backing.buffer as *const u8, backing.size);
    if !dst_buf.is_null() {
        (*dst_buf).buf_size = 0;
    }

    crate::udi_impl::buf::write(&mut dst_buf, 0..0, &data[src_off..][..src_len]);

    callback(gcb, dst_buf);
}