use super::*;

extern "C" {
    pub fn udi_endian_swap(
        src: *const c_void,
        dst: *mut c_void,
        swap_size: udi_ubit8_t,
        stride: udi_ubit8_t,
        rep_count: udi_ubit16_t);
}