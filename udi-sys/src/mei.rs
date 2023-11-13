//! Metalanguage-to-Environment Interface
use super::*;

/// This structure contains information describing the metalanguage-specific
/// properties of control blocks and ops vectors used with the particular
/// metalanguage. The environment uses this information to initialize drivers that
/// use each metalanguage, before executing any code in either driver or
/// metalanguage library.
#[repr(C)]
pub struct udi_mei_init_t {
    pub ops_vec_template_list: *const udi_mei_ops_vec_template_t,
    pub mei_enumeration_rank: *const udi_mei_enumeration_rank_func_t,
}

#[repr(C)]
pub struct udi_mei_ops_vec_template_t {
    /// `meta_ops_num`` is a number that identifies this ops vector type with respect
    /// to others in this metalanguage, or zero to terminate the
    /// `ops_vec_template_list` array to which this structure
    /// belongs (see [udi_mei_init_t]). If `meta_ops_num`` is zero,
    /// all other members of this structure are ignored.
    pub meta_ops_num: udi_index_t,
    /// `relationship`` defines the valid relationships between the regions on
    /// opposite ends of a channel when using an ops vector of this type.
    pub relationship: udi_ubit8_t,
    pub op_template_list: *const udi_mei_op_template_t,
}

pub const UDI_MEI_REL_INITIATOR: udi_ubit8_t = 1<<0;
pub const UDI_MEI_REL_BIND     : udi_ubit8_t = 1<<1;
pub const UDI_MEI_REL_EXTERNAL : udi_ubit8_t = 1<<2;
pub const UDI_MEI_REL_INTERNAL : udi_ubit8_t = 1<<3;
pub const UDI_MEI_REL_SINGLE   : udi_ubit8_t = 1<<4;


#[repr(C)]
pub struct udi_mei_op_template_t {
    op_name: *const ::core::ffi::c_char,
    op_category: udi_ubit8_t,
    op_flags: udi_ubit8_t,
    meta_cb_num: udi_index_t,
    completion_ops_num: udi_index_t,
    completion_vec_idx: udi_index_t,
    exception_ops_num: udi_index_t,
    exception_vec_idx: udi_index_t,
    direct_stub: *mut udi_mei_direct_stub_t,
    backend_stub: *mut udi_mei_backend_stub_t,
    visible_layout: *mut udi_layout_t,
    marshal_layout: *mut udi_layout_t,
}

/* Values for op_category */
pub const UDI_MEI_OPCAT_REQ: udi_ubit8_t = 1;
pub const UDI_MEI_OPCAT_ACK: udi_ubit8_t = 2;
pub const UDI_MEI_OPCAT_NAK: udi_ubit8_t = 3;
pub const UDI_MEI_OPCAT_IND: udi_ubit8_t = 4;
pub const UDI_MEI_OPCAT_RES: udi_ubit8_t = 5;
pub const UDI_MEI_OPCAT_RDY: udi_ubit8_t = 6;

/* Values for op_flags */
pub const UDI_MEI_OP_ABORTABLE    : udi_ubit8_t = 1<<0;
pub const UDI_MEI_OP_RECOVERABLE  : udi_ubit8_t = 1<<1;
pub const UDI_MEI_OP_STATE_CHANGE : udi_ubit8_t = 1<<2;
/* Maximum Sizes For Control Block Layouts */
pub const UDI_MEI_MAX_VISIBLE_SIZE: udi_size_t = 2000;
pub const UDI_MEI_MAX_MARSHAL_SIZE: udi_size_t = 4000;

pub type udi_mei_direct_stub_t = unsafe extern "C" fn(op: udi_op_t, gcb: *mut udi_cb_t, arglist: ::core::ffi::VaList);
pub type udi_mei_backend_stub_t = unsafe extern "C" fn(op: udi_op_t, gcb: *mut udi_cb_t, marshal_space: *mut c_void);
pub type udi_mei_enumeration_rank_func_t = unsafe extern "C" fn(attr_device_match: udi_ubit32_t, attr_value_list: *mut *mut c_void) -> udi_ubit8_t;

extern "C" {
    pub fn udi_mei_call(gcb: *mut udi_cb_t, meta_info: *mut udi_mei_init_t, meta_ops_num: udi_index_t, vec_idx: udi_index_t, ...);
    pub fn udi_mei_driver_error(gcb: *mut udi_cb_t, meta_info: *mut udi_mei_init_t, meta_ops_num: udi_index_t, vec_idx: udi_index_t);
}
