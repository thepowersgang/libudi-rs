
pub unsafe fn iter_with_layout<'a,'b>(layout: &'a *const ::udi::ffi::udi_layout_t, buffer: &'b mut *mut ::udi::ffi::c_void) -> DataIter<'a, 'b> {
    DataIter { layout: *layout, ptr: *buffer, _pd: ::core::marker::PhantomData }
}

pub struct DataIter<'a, 'data> {
    layout: *const ::udi::ffi::udi_layout_t,
    ptr: *mut ::udi::ffi::c_void,
    _pd: ::core::marker::PhantomData<(&'a ::udi::ffi::udi_layout_t, &'data mut ::udi::ffi::c_void)>,
}
impl<'a, 'data> DataIter<'a, 'data> {
    fn next_layout(&mut self) -> ::udi::ffi::udi_layout_t {
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
            _pd: std::marker::PhantomData,
        };
        while let Some(_) = rv1.next() {
        }

        // Create a new nested one (to return)
        let rv = DataIter {
            layout: self.layout,
            ptr: self.ptr,
            _pd: std::marker::PhantomData,
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
        ::udi::ffi::layout::UDI_DL_END => {
            // SAFE: Undoing the offset in `next_layout`
            self.layout = unsafe { self.layout.offset(-1) };
            return None
        },
        ::udi::ffi::layout::UDI_DL_UBIT8_T  => LayoutItem::UBit8 (self.advance()),
        ::udi::ffi::layout::UDI_DL_SBIT8_T  => LayoutItem::SBit8 (self.advance()),
        ::udi::ffi::layout::UDI_DL_UBIT16_T => LayoutItem::UBit16(self.advance()),
        ::udi::ffi::layout::UDI_DL_SBIT16_T => LayoutItem::SBit16(self.advance()),
        ::udi::ffi::layout::UDI_DL_UBIT32_T => LayoutItem::UBit32(self.advance()),
        ::udi::ffi::layout::UDI_DL_SBIT32_T => LayoutItem::SBit32(self.advance()),
        ::udi::ffi::layout::UDI_DL_BOOLEAN_T => LayoutItem::Boolean(self.advance()),

        ::udi::ffi::layout::UDI_DL_INDEX_T => LayoutItem::Index(self.advance()),

        ::udi::ffi::layout::UDI_DL_CHANNEL_T => LayoutItem::Channel(self.advance()),
        ::udi::ffi::layout::UDI_DL_ORIGIN_T => LayoutItem::Origin(self.advance()),

        ::udi::ffi::layout::UDI_DL_BUF => {
            let preserve_flag_ofs = self.next_layout();
            let preserve_flag_mask = self.next_layout();
            let preserve_flag_match = self.next_layout();
            LayoutItem::Buf(self.advance(), BufPreserveFlag(preserve_flag_ofs, preserve_flag_mask, preserve_flag_match))
            },
        ::udi::ffi::layout::UDI_DL_CB                 => LayoutItem::Cb(self.advance()),
        ::udi::ffi::layout::UDI_DL_INLINE_UNTYPED     => LayoutItem::InlineUntyped(self.advance()),
        ::udi::ffi::layout::UDI_DL_INLINE_DRIVER_TYPED=> LayoutItem::InlineDriverTyped(self.advance()),
        ::udi::ffi::layout::UDI_DL_MOVABLE_UNTYPED    => LayoutItem::InlineMovableUntyped(self.advance()),
        /* Nested Element Layout Type Codes */
        ::udi::ffi::layout::UDI_DL_INLINE_TYPED   => {
            let p = self.advance();
            let inner_layout = self.nested();
            LayoutItem::InlineTyped(p, inner_layout)
        },
        ::udi::ffi::layout::UDI_DL_MOVABLE_TYPED  => {
            let p = self.advance();
            let inner_layout = self.nested();
            LayoutItem::InlineMovableTyped(p, inner_layout)
        },
        ::udi::ffi::layout::UDI_DL_ARRAY => {
            let p = self.ptr;
            let count = self.next_layout();
            let inner_layout = self.nested();
            LayoutItem::Array(p, count, inner_layout)
        },

        _ => return None,  // Probably shouldn't happen, but here for completeness
        })
    }
    
}

pub enum LayoutItem<'layout, 'data>
{
    UBit8(&'data mut ::udi::ffi::udi_ubit8_t),
    SBit8(&'data mut ::udi::ffi::udi_sbit8_t),
    UBit16(&'data mut ::udi::ffi::udi_ubit16_t),
    SBit16(&'data mut ::udi::ffi::udi_sbit16_t),
    UBit32(&'data mut ::udi::ffi::udi_ubit32_t),
    SBit32(&'data mut ::udi::ffi::udi_sbit32_t),
    Boolean(&'data mut ::udi::ffi::udi_boolean_t),

    Index(&'data mut ::udi::ffi::udi_index_t),

    Channel(&'data mut ::udi::ffi::udi_channel_t),
    Origin(&'data mut ::udi::ffi::udi_origin_t),

    Buf(&'data mut *mut ::udi::ffi::udi_buf_t, BufPreserveFlag),
    Cb(&'data mut *mut ::udi::ffi::udi_cb_t),

    InlineUntyped(&'data mut *mut ::udi::ffi::c_void),
    InlineDriverTyped(&'data mut *mut ::udi::ffi::c_void),
    InlineMovableUntyped(&'data mut *mut ::udi::ffi::c_void),

    InlineTyped(&'data mut *mut ::udi::ffi::c_void, DataIter<'layout, 'data>),
    InlineMovableTyped(&'data mut *mut ::udi::ffi::c_void, DataIter<'layout, 'data>),
    Array(*mut ::udi::ffi::c_void, u8, DataIter<'layout, 'data>)
}

// TODO: To check this, the layout is needed
pub struct BufPreserveFlag(u8,u8,u8);
impl BufPreserveFlag {
    //pub unsafe fn test(cb: *const udi_cb_t) -> bool {
    //}
}