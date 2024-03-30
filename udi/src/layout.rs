//! UDI structure/memory layout handling

/// Iterate a buffer using a provided layout to provide the data structure
pub unsafe fn iter_with_layout<'a,'b>(layout: &'a *const crate::ffi::udi_layout_t, buffer: &'b mut *mut crate::ffi::c_void) -> DataIter<'a, 'b> {
    DataIter { layout: *layout, ptr: *buffer, _pd: ::core::marker::PhantomData }
}

/// Iterator over a buffer using a layout (see [iter_with_layout])
pub struct DataIter<'a, 'data> {
    layout: *const crate::ffi::udi_layout_t,
    ptr: *mut crate::ffi::c_void,
    _pd: ::core::marker::PhantomData<(&'a crate::ffi::udi_layout_t, &'data mut crate::ffi::c_void)>,
}
impl<'a, 'data> DataIter<'a, 'data> {
    fn next_layout(&mut self) -> crate::ffi::udi_layout_t {
        unsafe {
            let rv = *self.layout;
            self.layout = self.layout.offset(1);
            rv
        }
    }
    fn advance<T>(&mut self) -> &'data mut T {
        // SAFE: Trusting the constructor of this type to ensure that we don't go out of bounds
        unsafe {
            let rv = self.ptr as *mut T;
            self.ptr = self.ptr.offset(::core::mem::size_of::<T>() as _);
            &mut *rv
        }
    }

    fn nested(&mut self) -> DataIter<'a, 'data> {
        // Create a nested version, and iterate it until it hits END
        let mut rv1 = DataIter {
            layout: self.layout,
            ptr: self.ptr,
            _pd: ::core::marker::PhantomData,
        };
        while let Some(_) = rv1.next() {
        }

        // Create a new nested one (to return)
        let rv = DataIter {
            layout: self.layout,
            ptr: self.ptr,
            _pd: ::core::marker::PhantomData,
        };

        // Update our state based on the exhausted one above
        // SAFE: This is re-doing the offset in `next` when END is hits
        self.layout = unsafe { rv1.layout.offset(1) };
        self.ptr = rv1.ptr;

        rv
    }
}
impl<'layout, 'data> Iterator for DataIter<'layout, 'data>
{
    type Item = LayoutItem<'layout, 'data>;

    fn next(&mut self) -> Option<Self::Item>
    {
        Some(match self.next_layout()
        {
        crate::ffi::layout::UDI_DL_END => {
            // SAFE: Undoing the offset in `next_layout`
            self.layout = unsafe { self.layout.offset(-1) };
            return None
        },
        crate::ffi::layout::UDI_DL_UBIT8_T  => LayoutItem::UBit8 (self.advance()),
        crate::ffi::layout::UDI_DL_SBIT8_T  => LayoutItem::SBit8 (self.advance()),
        crate::ffi::layout::UDI_DL_UBIT16_T => LayoutItem::UBit16(self.advance()),
        crate::ffi::layout::UDI_DL_SBIT16_T => LayoutItem::SBit16(self.advance()),
        crate::ffi::layout::UDI_DL_UBIT32_T => LayoutItem::UBit32(self.advance()),
        crate::ffi::layout::UDI_DL_SBIT32_T => LayoutItem::SBit32(self.advance()),
        crate::ffi::layout::UDI_DL_BOOLEAN_T => LayoutItem::Boolean(self.advance()),

        crate::ffi::layout::UDI_DL_INDEX_T => LayoutItem::Index(self.advance()),

        crate::ffi::layout::UDI_DL_CHANNEL_T => LayoutItem::Channel(self.advance()),
        crate::ffi::layout::UDI_DL_ORIGIN_T => LayoutItem::Origin(self.advance()),

        crate::ffi::layout::UDI_DL_BUF => {
            let preserve_flag_ofs = self.next_layout();
            let preserve_flag_mask = self.next_layout();
            let preserve_flag_match = self.next_layout();
            LayoutItem::Buf(self.advance(), BufPreserveFlag(preserve_flag_ofs, preserve_flag_mask, preserve_flag_match))
            },
        crate::ffi::layout::UDI_DL_CB                 => LayoutItem::Cb(self.advance()),
        crate::ffi::layout::UDI_DL_INLINE_UNTYPED     => LayoutItem::InlineUntyped(self.advance()),
        crate::ffi::layout::UDI_DL_INLINE_DRIVER_TYPED=> LayoutItem::InlineDriverTyped(self.advance()),
        crate::ffi::layout::UDI_DL_MOVABLE_UNTYPED    => LayoutItem::InlineMovableUntyped(self.advance()),
        /* Nested Element Layout Type Codes */
        crate::ffi::layout::UDI_DL_INLINE_TYPED   => {
            let p = self.advance();
            let inner_layout = self.nested();
            LayoutItem::InlineTyped(p, inner_layout)
        },
        crate::ffi::layout::UDI_DL_MOVABLE_TYPED  => {
            let p = self.advance();
            let inner_layout = self.nested();
            LayoutItem::InlineMovableTyped(p, inner_layout)
        },
        crate::ffi::layout::UDI_DL_ARRAY => {
            let p = self.ptr;
            let count = self.next_layout();
            let inner_layout = self.nested();
            LayoutItem::Array(p, count, inner_layout)
        },

        _ => return None,  // Probably shouldn't happen, but here for completeness
        })
    }
    
}

/// A data element decoded using a layout
pub enum LayoutItem<'layout, 'data>
{
    /// 8 bit unsigned (`udi_ubit8_t`)
    UBit8(&'data mut crate::ffi::udi_ubit8_t),
    /// 8 bit signed (`udi_sbit8_t`)
    SBit8(&'data mut crate::ffi::udi_sbit8_t),
    /// 16 bit unsigned (`udi_ubit16_t`)
    UBit16(&'data mut crate::ffi::udi_ubit16_t),
    /// 16 bit signed (`udi_sbit16_t`)
    SBit16(&'data mut crate::ffi::udi_sbit16_t),
    /// 32 bit unsigned (`udi_ubit32_t`)
    UBit32(&'data mut crate::ffi::udi_ubit32_t),
    /// 32 bit signed (`udi_sbit32_t`)
    SBit32(&'data mut crate::ffi::udi_sbit32_t),
    /// Boolean value
    Boolean(&'data mut crate::ffi::udi_boolean_t),

    /// An 8-bit index
    Index(&'data mut crate::ffi::udi_index_t),

    /// A channel handle
    Channel(&'data mut crate::ffi::udi_channel_t),
    /// An origin handle
    Origin(&'data mut crate::ffi::udi_origin_t),

    /// Another buffer
    Buf(&'data mut *mut crate::ffi::udi_buf_t, BufPreserveFlag),
    /// A control block
    Cb(&'data mut *mut crate::ffi::udi_cb_t),

    /// Inline data pointer with no associated type information
    InlineUntyped(&'data mut *mut crate::ffi::c_void),
    /// Inline data pointer with a type provided by the driver
    InlineDriverTyped(&'data mut *mut crate::ffi::c_void),
    /// Inline and movable data pointer with no associated type information
    InlineMovableUntyped(&'data mut *mut crate::ffi::c_void),

    /// Inline data pointer with a known type (includes layout/iterator)
    InlineTyped(&'data mut *mut crate::ffi::c_void, DataIter<'layout, 'data>),
    /// Inline and movable data pointer with a known type (includes layout/iterator)
    InlineMovableTyped(&'data mut *mut crate::ffi::c_void, DataIter<'layout, 'data>),
    /// An array (includes layout/iterator)
    Array(*mut crate::ffi::c_void, u8, DataIter<'layout, 'data>)
}

// TODO: To check this, the layout is needed
/// Information required to know if a buffer needs to be preserved
pub struct BufPreserveFlag(u8,u8,u8);
impl BufPreserveFlag {
    //pub unsafe fn test(cb: *const udi_cb_t) -> bool {
    //}
}


/// Trait used to obtain the layout of a data type
pub unsafe trait GetLayout
{
    /// Length of the data
    const LEN: usize;
    /// Layout entries
    const LAYOUT: &'static [u8];
}
macro_rules! impl_layout_simple {
    ( $( $t:ty => $flag:ident, )+ ) => {
        $(
        unsafe impl GetLayout for $t {
            const LEN: usize = 1;
            const LAYOUT: &'static [u8] = &[crate::ffi::layout::$flag];
        }
        )+
    };
}
impl_layout_simple!{
    crate::ffi::udi_ubit8_t => UDI_DL_UBIT8_T,
    crate::ffi::udi_sbit8_t => UDI_DL_SBIT8_T,
    crate::ffi::udi_ubit16_t => UDI_DL_UBIT16_T,
    crate::ffi::udi_sbit16_t => UDI_DL_SBIT16_T,
    crate::ffi::udi_ubit32_t => UDI_DL_UBIT32_T,
    crate::ffi::udi_sbit32_t => UDI_DL_SBIT32_T,
    crate::ffi::udi_boolean_t => UDI_DL_BOOLEAN_T,

    crate::ffi::udi_index_t => UDI_DL_INDEX_T,
    crate::ffi::udi_channel_t => UDI_DL_CHANNEL_T,
    crate::ffi::udi_origin_t => UDI_DL_ORIGIN_T,
    // The rest are more complex, so are handled in the derive
}

