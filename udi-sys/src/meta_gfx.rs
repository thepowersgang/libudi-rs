//! UDI GFX Bindings (draft)
//! 
// From public domain header by "Marcel Sondaar"
// See https://mysticos.combuster.nl/downloads/mos-nightly-source.tar.gz
// - mos/include/common/udi_gfx.h

use crate::{udi_cb_t, udi_status_t, udi_buf_t};
use crate::{udi_index_t, udi_ubit32_t};

pub macro metalang_name( $($prefix:ident::)* ) {
    $($prefix::)*udi_gfx
}

macro_rules! a {
    ( $( $(#[$a:meta])* $op_name:ident = fn $name:ident($($a_name:ident: $a_ty:ty),* $(,)?); )+ ) => {
        extern "C" {
            $( $(#[$a])* pub fn $name( $($a_name: $a_ty),* ); )+
        }
        $( pub type $op_name = unsafe extern "C" fn ( $($a_name: $a_ty),* ); )+
    };
}

// --- Properties
pub use _UDI_GFX_PROP::*;
#[derive(Copy,Clone)]
#[repr(u32)]
/**
 * Lists the various UDI properties
 */
pub enum _UDI_GFX_PROP {
    /**
     * The primary state of the connector or engine. An enabled state indicates
     * it is functioning and generating live output. A disabled state is one where
     * it is not contributing to any output but is otherwise functional. Finally
     * the reset state is where the driver is free to deallocate all resources 
     * corresponding to this component and trash any state not referenced by other
     * components.
     *
     * A disabled or reset engine forwards all data from the previous stage 
     * unmodified. The disabled state indicates that the component might be 
     * returned to its enabled state within short notice.
     *
     * A disabled connector will not send pixel data, but can perform other 
     * initialisation communication such as DDC. A reset connector will not respond
     * in any fashion and can not be used for other purposes. Hardware is expected
     * to be powered down in such state.
     *
     * Users should expect significant delays when moving components in and out of
     * the reset state. Moving engines between the enabled and disabled state should
     * take effect within one frame, such transition should take effect on a frame 
     * boundary when supported.
     * 
     * Valid values:
     *     0 - disabled
     *     1 - enabled
     *     2 - reset
     *
     * Ranges:
     *     Must include at least 1
     */
    UDI_GFX_PROP_ENABLE           =  0,
    /**
     * Points to the engine that is processed before this unit. In the case of a 
     * connector, it points to the last engine in a pipeline, and each engine points 
     * to the next engine in the sequence. A value of -1 indicates a source that 
     * only yields black pixels. Implementations must not allow cyclic structures. 
     * Changing this value may reallocate resources, and engines that are no longer 
     * referenced may lose their data (but not their state) when it is not part of 
     * any pipeline. If preservation is required, the ENABLE state should be used
     * instead. Valid ranges includes one or more from the list of engines and -1 
     * combined. In most cases, this property can not be modified.
     * 
     * Valid values:
     *     Any valid engine ID, provided no dependency cycles are created, or -1
     *
     * Ranges:
     *     Any non-empty set of valid values. Often hardwired.
     *
     */
    UDI_GFX_PROP_INPUT            =  1,
    /**
     * Contains the amount of pixels in the horizontal direction. For connectors, 
     * this is the amount of data pixels rendered horizontally. For engines, this 
     * is the width in pixels of the image. Pixels requested from an engine outside 
     * the range (0..width-1) are defined according to the <UDI_GFX_PROP_CLIP> 
     * property. In some cases, hardware may support only fixed combinations of 
     * width and height. In such cases, changing the width will also change the 
     * height to a corresponding valid number. Valid ranges include any values
     * strictly above zero. For connectors, expect large continuous ranges, large
     * ranges with a certain modulus, a limited number of fixed values, or a
     * constant value.
     * 
     * Valid values:
     *     Any non-zero positive number.
     *
     * Ranges:
     *     Contains at least one valid value. Often only multiples of UNIT_WIDTH
     *     or a power of two are allowed. May be hardwired.
     */
    UDI_GFX_PROP_WIDTH            =  2,
    UDI_GFX_PROP_HEIGHT           =  3,
    UDI_GFX_PROP_CUSTOM           = 1024,
    UDI_GFX_PROP_CLIP             =  4,
    UDI_GFX_PROP_UNIT_WIDTH       =  5,
    UDI_GFX_PROP_UNIT_HEIGHT      =  6,
    UDI_GFX_PROP_TRANSLATEX       =  7,
    UDI_GFX_PROP_TRANSLATEY       =  8,
    UDI_GFX_PROP_GL_VERSION       = 14,
    UDI_GFX_PROP_GLES_VERSION     = 15,
    UDI_GFX_PROP_STATE_BLOCK      = 16,
    UDI_GFX_PROP_COLOR_BITS       = 22,
    // Duplicates?
    //UDI_GFX_PROP_GL_TARGET        = 23,
    //UDI_GFX_PROP_STOCK_FORMAT     = 27,
    //UDI_GFX_PROP_OPERATOR_COUNT   = 28,

    // Deprecated items
    //UDI_GFX_PROP_STORE_COUNT       24,     /* not generic*/
    //UDI_GFX_PROP_STORE_WIDTH        9,     /* not generic*/
    //UDI_GFX_PROP_STORE_HEIGHT      10,     /* not generic*/
    //UDI_GFX_PROP_STORE_BITS        11,     /* not generic*/
    //UDI_GFX_PROP_PALETTE           1024,   /* optional, can be derived from the operator tree*/
    //UDI_GFX_PROP_BUFFER            1025,   /* optional, can be derived from the operator tree*/
    //UDI_GFX_PROP_TILESHEET         1026,   /* optional, can be derived from the operator tree*/
    //UDI_GFX_PROP_OPERATOR_INDEX    17,     /* deprecated for dedicated methods*/
    //UDI_GFX_PROP_OPERATOR_OPCODE   18,     /* deprecated for dedicated methods*/
    //UDI_GFX_PROP_OPERATOR_ARG_1    19,     /* deprecated for dedicated methods*/
    //UDI_GFX_PROP_OPERATOR_ARG_2    20,     /* deprecated for dedicated methods*/
    //UDI_GFX_PROP_OPERATOR_ARG_3    21,     /* deprecated for dedicated methods*/
    //UDI_GFX_PROP_SOURCE_WIDTH      12,     /* should have been documented when I still knew what it did.*/
    //UDI_GFX_PROP_SOURCE_HEIGHT     13,     /* should have been documented when I still knew what it did.*/
    //UDI_GFX_PROP_INPUTX            25,     /* should have been documented when I still knew what it did.*/
    //UDI_GFX_PROP_INPUTY            26,     /* should have been documented when I still knew what it did.*/
    UDI_GFX_PROP_SIGNAL           = 23,
    UDI_GFX_PROP_CONNECTOR_TYPE   = 24,
    UDI_GFX_PROP_VGA_H_FRONT_PORCH= 25,
    UDI_GFX_PROP_VGA_H_BACK_PORCH = 26,
    UDI_GFX_PROP_VGA_H_SYNC       = 27,
    UDI_GFX_PROP_VGA_V_FRONT_PORCH= 28,
    UDI_GFX_PROP_VGA_V_BACK_PORCH = 29,
    UDI_GFX_PROP_VGA_V_SYNC       = 30,
    UDI_GFX_PROP_DOT_CLOCK        = 31,
    UDI_GFX_PROP_VGA_H_SYNC_POL   = 32,
    UDI_GFX_PROP_VGA_V_SYNC_POL   = 33,
}

pub use _UDI_GFX_PROP_ENABLE::*;
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum _UDI_GFX_PROP_ENABLE {
    UDI_GFX_PROP_ENABLE_DISABLED  = 0,
    UDI_GFX_PROP_ENABLE_ENABLED   = 1,
    UDI_GFX_PROP_ENABLE_RESET     = 2,
}

pub use _UDI_GFX_SIGNAL::*;
/**
 * Lists the various signal types
 */
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum _UDI_GFX_SIGNAL {
    //UDI_GFX_SIGNAL_HIDDEN     = 0,    // Duplicate
    UDI_GFX_SIGNAL_INTEGRATED = 0,
    UDI_GFX_SIGNAL_RGBHV      = 1,
    UDI_GFX_SIGNAL_RGBS       = 2,
    UDI_GFX_SIGNAL_RGSB       = 3,
    UDI_GFX_SIGNAL_YPBPR      = 4,
    UDI_GFX_SIGNAL_DVID       = 5,
    UDI_GFX_SIGNAL_YUV        = 6,
    UDI_GFX_SIGNAL_YIQ        = 7,
    UDI_GFX_SIGNAL_Y_UV       = 8,
    UDI_GFX_SIGNAL_Y_IQ       = 9,
    UDI_GFX_SIGNAL_HDMI       = 10,
    UDI_GFX_SIGNAL_TEXT       = 11,
    UDI_GFX_SIGNAL_CUSTOM     = 12,
}

pub use _UDI_GFX_CONNECTOR::*;
/**
 * Lists the various external connectors
 */
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum _UDI_GFX_CONNECTOR {
    UDI_GFX_CONNECTOR_HIDDEN    = 0,
    UDI_GFX_CONNECTOR_VGA       = 1,
    UDI_GFX_CONNECTOR_DVI       = 2,
    UDI_GFX_CONNECTOR_SVIDEO    = 3,
    UDI_GFX_CONNECTOR_COMPONENT = 4,
    UDI_GFX_CONNECTOR_HDMI      = 5,
    UDI_GFX_CONNECTOR_RF        = 6,
    UDI_GFX_CONNECTOR_SCART     = 7,
    UDI_GFX_CONNECTOR_COMPOSITE = 8,
    UDI_GFX_CONNECTOR_MEMBUFFER = 9,
}

pub use _UDI_GFX_OPERATOR::*;
/**
 * Lists the display output operator
 */
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum _UDI_GFX_OPERATOR {
    /// `output = (color) red(a1) + green(a2) + blue(a3)` (each component is UDI_GFX_PROP_COLOR_BITS
    UDI_GFX_OPERATOR_RGB    =  0,
    /// `output = (color) Y(a1) + U(a2) + V(a3)`
    UDI_GFX_OPERATOR_YUV    =  1,
    /// `output = (color) Y(a1) + I(a2) + Q(a3)`
    UDI_GFX_OPERATOR_YIQ    =  2,
    /// `output = (color) intensity(a1)`
    UDI_GFX_OPERATOR_I      =  3,
    /// `output = (color) a1 + alpha(a2)`
    UDI_GFX_OPERATOR_ALPHA  =  4,
    /// `output = a1 + a2 + v3`
    UDI_GFX_OPERATOR_ADD    =  5,
    /// `output = a1 - a2 - v3`
    UDI_GFX_OPERATOR_SUB    =  6,
    /// `output = a1 * a2`
    UDI_GFX_OPERATOR_MUL    =  7,
    /// `output = a1 / a2`
    UDI_GFX_OPERATOR_DIV    =  8,
    /// `output = a1 * a2 + a3`
    UDI_GFX_OPERATOR_MAD    =  9,
    /// `output = (a1 * a2) / a3`
    UDI_GFX_OPERATOR_FRC    = 10,
    /// `output = a1 >> (a2 + v3)`
    UDI_GFX_OPERATOR_SHR    = 11,
    /// `output = a1 << (a2 + v3)`
    UDI_GFX_OPERATOR_SHL    = 12,
    /// `output = a1 >> a2` (over a3 bits)
    UDI_GFX_OPERATOR_ROR    = 13,
    /// `output = a1 << a2` (over a3 bits)
    UDI_GFX_OPERATOR_ROL    = 14,
    /// `output = a1 >> a2` (width is a3 bits, i.e. empties are filled with bit a3-1)
    UDI_GFX_OPERATOR_SAR    = 15,
    /// `output = a1 <<< (a2 + v3)` (empties filled with bit 0)
    UDI_GFX_OPERATOR_SAL    = 16,
    /// `output = a1 & a2`
    UDI_GFX_OPERATOR_AND    = 17,
    /// output = a1 | a2 | v3
    UDI_GFX_OPERATOR_OR     = 18,
    /// `output = ~a1`
    UDI_GFX_OPERATOR_NOT    = 19,
    /// `output = a1 ^ a2 ^ v3`
    UDI_GFX_OPERATOR_XOR    = 20,
    /// output = -a1
    UDI_GFX_OPERATOR_NEG    = 21,
    /// `output = (a1 >> v2) & (2**v3-1)` (select `v3` bits starting from bit `v2`)
    UDI_GFX_OPERATOR_SEG    = 22,
    /// `output = (a1 > a2) ? a2 : ((a1 < a3) ? a3 : a1)`
    UDI_GFX_OPERATOR_RANGE  = 23,
    /// `output = v1`
    UDI_GFX_OPERATOR_CONST  = 24,
    /// `output = property[a1 + v2]`
    UDI_GFX_OPERATOR_ATTR   = 25,
    /// `output = output[(a1 % v3) + v2]`
    UDI_GFX_OPERATOR_SWITCH = 26,
    /// `output = buffer[a1][a2]` (buffer is `v3` bits per entry)
    UDI_GFX_OPERATOR_BUFFER = 27,
    /// `output = output x pixel`
    UDI_GFX_OPERATOR_X      = 28,
    /// `output = output y pixel`
    UDI_GFX_OPERATOR_Y      = 29,
    /// `output = horizontal tile index belonging to output pixel`
    UDI_GFX_OPERATOR_TX     = 30,
    /// `output = vertical tile index belonging to output pixel`
    UDI_GFX_OPERATOR_TY     = 31,
    /// `output = horizontal offset from start of tile`
    UDI_GFX_OPERATOR_TXOFF  = 32,
    /// `output = vertical offset from start of tile`
    UDI_GFX_OPERATOR_TYOFF  = 33,
    /// `output = input engine[x][y]   component v1`
    UDI_GFX_OPERATOR_INPUT  = 34,
    /// `output = input engine[a1][a2] component v3`
    UDI_GFX_OPERATOR_DINPUT = 35,
}

pub use _UDI_GFX_STOCK_FORMAT::*;
/**
 * Lists stock configurations
 *
 * When a stock configuration is used, the device is set to behave as a 
 * simple framebuffer device. The [UDI_GFX_PROP_WIDTH] and [UDI_GFX_PROP_HEIGHT]
 * determine the virtual size of the framebuffer, and [UDI_GFX_PROP_TRANSLATEX]
 * and [UDI_GFX_PROP_TRANSLATEY] indicate the offset into that framebuffer 
 * that is visible (which are typically restricted to negative values)
 */
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum _UDI_GFX_STOCK_FORMAT {
    UDI_GFX_STOCK_FORMAT_UNKNOWN  = 0,
    UDI_GFX_STOCK_FORMAT_R8G8B8X8 = 1,
    UDI_GFX_STOCK_FORMAT_B8G8R8X8 = 2,
    UDI_GFX_STOCK_FORMAT_R8G8B8   = 3,
    UDI_GFX_STOCK_FORMAT_B8G8R8   = 4,
    UDI_GFX_STOCK_FORMAT_R5G6B5   = 5,
    UDI_GFX_STOCK_FORMAT_B5G6R5   = 6,
    UDI_GFX_STOCK_FORMAT_R5G5B5X1 = 7,
    UDI_GFX_STOCK_FORMAT_B5G5R5X1 = 8,
    UDI_GFX_STOCK_FORMAT_N8       = 9,
}

pub const UDI_GFX_BUFFER_INFO_FLAG_R             : u32 = 0x0001;  /* buffer can be read*/
pub const UDI_GFX_BUFFER_INFO_FLAG_W             : u32 = 0x0002;  /* buffer can be written*/
pub const UDI_GFX_BUFFER_INFO_FLAG_BITALIGN_ENTRY: u32 = 0x0004;  /* for non-multiple-of-eight buffer slot sizes, align on byte boundary every unit*/
pub const UDI_GFX_BUFFER_INFO_FLAG_BITALIGN_ROW  : u32 = 0x0008;  /* for non-multiple-of-eight buffer slot sizes, align only the start of the row*/

// ---


#[repr(C)]
pub struct udi_gfx_provider_ops_t {
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub gfx_bind_req_op: udi_gfx_bind_req_op_t,
    pub gfx_unbind_req_op: udi_gfx_unbind_req_op_t,
    pub gfx_set_connector_req_op: udi_gfx_set_connector_req_op_t,
    pub gfx_set_engine_req_op: udi_gfx_set_engine_req_op_t,
    pub gfx_get_connector_req_op: udi_gfx_get_connector_req_op_t,
    pub gfx_get_engine_req_op: udi_gfx_get_engine_req_op_t,
    pub gfx_range_connector_req_op: udi_gfx_range_connector_req_op_t,
    pub gfx_range_engine_req_op: udi_gfx_range_engine_req_op_t,
    pub gfx_get_engine_operator_req_op: udi_gfx_get_engine_operator_req_op_t,
    pub gfx_connector_command_req_op: udi_gfx_connector_command_req_op_t,
    pub gfx_engine_command_req_op: udi_gfx_engine_command_req_op_t,
    pub gfx_buffer_info_req_op: udi_gfx_buffer_info_req_op_t,
    pub gfx_buffer_read_req_op: udi_gfx_buffer_read_req_op_t,
    pub gfx_buffer_write_req_op: udi_gfx_buffer_write_req_op_t,
}
pub const UDI_GFX_PROVIDER_OPS_NUM: u8 = 1;

#[repr(C)]
pub struct udi_gfx_client_ops_t {
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub gfx_bind_ack_op: udi_gfx_bind_ack_op_t,
    pub gfx_unbind_ack_op: udi_gfx_unbind_ack_op_t,
    pub gfx_set_connector_ack_op: udi_gfx_set_connector_ack_op_t,
    pub gfx_set_engine_ack_op: udi_gfx_set_engine_ack_op_t,
    pub gfx_set_connector_nak_op: udi_gfx_set_connector_nak_op_t,
    pub gfx_set_engine_nak_op: udi_gfx_set_engine_nak_op_t,
    pub gfx_get_connector_ack_op: udi_gfx_get_connector_ack_op_t,
    pub gfx_get_engine_ack_op: udi_gfx_get_engine_ack_op_t,
    pub gfx_range_connector_ack_op: udi_gfx_range_connector_ack_op_t,
    pub gfx_range_engine_ack_op: udi_gfx_range_engine_ack_op_t,
    pub gfx_get_engine_operator_ack_op: udi_gfx_get_engine_operator_req_op_t,
    pub gfx_connector_command_ack_op: udi_gfx_connector_command_ack_op_t,
    pub gfx_engine_command_ack_op: udi_gfx_engine_command_ack_op_t,
    pub gfx_buffer_info_ack_op: udi_gfx_buffer_info_ack_op_t,
    pub gfx_buffer_read_ack_op: udi_gfx_buffer_read_ack_op_t,
    pub gfx_buffer_write_ack_op: udi_gfx_buffer_write_ack_op_t,
    pub gfx_buffer_read_nak_op: udi_gfx_buffer_read_nak_op_t,
    pub gfx_buffer_write_nak_op: udi_gfx_buffer_write_nak_op_t,
}
pub const UDI_GFX_CLIENT_OPS_NUM: u8 = 1;

#[repr(C)]
/// Contains the operations of a driver binding request
pub struct udi_gfx_bind_cb_t {
    pub gcb: udi_cb_t,
}
pub const UDI_GFX_BIND_CB_NUM: u8 = 1;
a!{
    udi_gfx_bind_req_op_t = fn udi_gfx_bind_req(cb: *mut udi_gfx_bind_cb_t);
    udi_gfx_bind_ack_op_t = fn udi_gfx_bind_ack(
        cb: *mut udi_gfx_bind_cb_t,
        sockets: udi_index_t,
        engines: udi_index_t,
        status: udi_status_t,
    );
    udi_gfx_unbind_req_op_t = fn udi_gfx_unbind_req(cb: *mut udi_gfx_bind_cb_t);
    udi_gfx_unbind_ack_op_t = fn udi_gfx_unbind_ack(cb: *mut udi_gfx_bind_cb_t);
}


#[repr(C)]
/// Contains the operations of a read/write transaction
pub struct udi_gfx_state_cb_t {
    pub gcb: udi_cb_t,
    pub subsystem: udi_ubit32_t,
    pub attribute: udi_ubit32_t,
}
pub const UDI_GFX_STATE_CB_NUM: u8 = 2;
a!{
    udi_gfx_set_engine_req_op_t     = fn udi_gfx_set_engine_req(cb: *mut udi_gfx_state_cb_t, value: udi_ubit32_t);
    udi_gfx_set_connector_req_op_t  = fn udi_gfx_set_connector_req (cb: *mut udi_gfx_state_cb_t, value: udi_ubit32_t);
    udi_gfx_set_engine_ack_op_t     = fn udi_gfx_set_engine_ack(cb: *mut udi_gfx_state_cb_t);
    udi_gfx_set_connector_ack_op_t  = fn udi_gfx_set_connector_ack (cb: *mut udi_gfx_state_cb_t);
    udi_gfx_get_engine_req_op_t     = fn udi_gfx_get_engine_req(cb: *mut udi_gfx_state_cb_t);
    udi_gfx_get_connector_req_op_t  = fn udi_gfx_get_connector_req (cb: *mut udi_gfx_state_cb_t);
    udi_gfx_get_engine_ack_op_t     = fn udi_gfx_get_engine_ack(cb: *mut udi_gfx_state_cb_t, value: udi_ubit32_t);
    udi_gfx_get_connector_ack_op_t  = fn udi_gfx_get_connector_ack (cb: *mut udi_gfx_state_cb_t, value: udi_ubit32_t);
    udi_gfx_set_engine_nak_op_t     = fn udi_gfx_set_engine_nak(cb: *mut udi_gfx_state_cb_t, status: udi_status_t);
    udi_gfx_get_connector_nak_op_t  = fn udi_gfx_get_connector_nak (cb: *mut udi_gfx_state_cb_t, status: udi_status_t);
    udi_gfx_set_connector_nak_op_t  = fn udi_gfx_set_connector_nak(cb: *mut udi_gfx_state_cb_t, status: udi_status_t);
}

#[repr(C)]
/// Contains the operations of a range request transaction
pub struct udi_gfx_range_cb_t {
    pub gcb: udi_cb_t,
    pub subsystem: udi_ubit32_t,
    pub attribute: udi_ubit32_t,
    pub rangedata: *mut udi_buf_t,
}
pub const UDI_GFX_RANGE_CB_NUM: u8 = 3;
a!{
    udi_gfx_range_engine_req_op_t    = fn udi_gfx_range_engine_req(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_range_connector_req_op_t = fn udi_gfx_range_connector_req(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_range_engine_ack_op_t    = fn udi_gfx_range_engine_ack(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_range_connector_ack_op_t = fn udi_gfx_range_connector_ack(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_get_engine_operator_req_op_t = fn udi_gfx_get_engine_operator_req(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_get_engine_operator_ack_op_t = fn udi_gfx_get_engine_operator_ack(
        cb: *mut udi_gfx_range_cb_t,
        op: udi_ubit32_t,
        arg1: udi_ubit32_t,
        arg2: udi_ubit32_t,
        arg3: udi_ubit32_t,
    );
}


#[repr(C)]
/// Contains the operations of a command sequence
pub struct udi_gfx_command_cb_t {
    pub gcb: udi_cb_t,
    pub commanddata: *mut udi_buf_t,
}
pub const UDI_GFX_COMMAND_CB_NUM: u8 = 4;
a!{
    udi_gfx_connector_command_req_op_t  = fn udi_gfx_connector_command_req(cb: *mut udi_gfx_command_cb_t);
    udi_gfx_engine_command_req_op_t     = fn udi_gfx_engine_command_req(cb: *mut udi_gfx_command_cb_t);
    udi_gfx_connector_command_ack_op_t  = fn udi_gfx_connector_command_ack(cb: *mut udi_gfx_command_cb_t);
    udi_gfx_engine_command_ack_op_t     = fn udi_gfx_engine_command_ack(cb: *mut udi_gfx_command_cb_t);
}

#[repr(C)]
// /// Contains a description of a buffer, or area thereof
pub struct udi_gfx_buffer_info_cb_t {
    pub gcb: udi_cb_t,
    pub buffer_index: udi_ubit32_t,
}
//pub const UDI_GFX_BUFFER_INFO_CB_NUM: u8 = 5;
a!{
    udi_gfx_buffer_info_req_op_t = fn udi_gfx_buffer_info_req(cb: *mut udi_gfx_buffer_info_cb_t);
    udi_gfx_buffer_info_ack_op_t = fn udi_gfx_buffer_info_ack(
        cb: *mut udi_gfx_buffer_info_cb_t,
        width: udi_ubit32_t,
        height: udi_ubit32_t,
        bitsper: udi_ubit32_t,
        flags: udi_ubit32_t,
    );
}


#[repr(C)]
/// Contains a description of a buffer, or area thereof
pub struct udi_gfx_buffer_cb_t {
    pub gcb: udi_cb_t,
    pub buffer_index: udi_ubit32_t,
    pub x: udi_ubit32_t,
    pub y: udi_ubit32_t,
    pub width: udi_ubit32_t,
    pub height: udi_ubit32_t,
    pub buffer: *mut udi_buf_t,
}
//pub const UDI_GFX_BUFFER_CB_NUM: u8 = 6;
a!{
    udi_gfx_buffer_write_req_op_t = fn udi_gfx_buffer_write_req(cb: *mut udi_gfx_buffer_cb_t);
    udi_gfx_buffer_read_req_op_t  = fn udi_gfx_buffer_read_req (cb: *mut udi_gfx_buffer_cb_t);
    udi_gfx_buffer_write_ack_op_t = fn udi_gfx_buffer_write_ack(cb: *mut udi_gfx_buffer_cb_t);
    udi_gfx_buffer_read_ack_op_t  = fn udi_gfx_buffer_read_ack (cb: *mut udi_gfx_buffer_cb_t);
    udi_gfx_buffer_write_nak_op_t = fn udi_gfx_buffer_write_nak(cb: *mut udi_gfx_buffer_cb_t, status: udi_status_t);
    udi_gfx_buffer_read_nak_op_t  = fn udi_gfx_buffer_read_nak (cb: *mut udi_gfx_buffer_cb_t, status: udi_status_t);
}

