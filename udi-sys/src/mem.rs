
pub type udi_mem_alloc_call_t = unsafe extern "C" fn(gcb: *mut super::udi_cb_t, new_mem: *mut super::c_void);

extern "C" {
    pub fn udi_mem_alloc(callback: udi_mem_alloc_call_t, gcb: *mut super::udi_cb_t, size: super::udi_size_t, flags: super::udi_ubit8_t);
    pub fn udi_mem_free(target_mem: *mut super::c_void);
}

pub const UDI_MEM_NOZERO: super::udi_ubit8_t = 1 << 0;
pub const UDI_MEM_MOVABLE: super::udi_ubit8_t = 1 << 0;
