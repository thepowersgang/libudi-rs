use ::udi::ffi::*;

#[derive(Default,Clone)]
pub struct ConstaintsReal {
    attrs: Vec<physio::udi_dma_constraints_attr_spec_t>,
}
impl ConstaintsReal {
    pub unsafe fn from_ref(constraints: &physio::udi_dma_constraints_t) -> &Self {
        &*(*constraints as *const ConstaintsReal)
    }

    pub fn get(&self, attr_type: physio::udi_dma_constraints_attr_t) -> Option<u32> {
        match self.attrs.binary_search_by_key(&attr_type, |v| v.attr_type)
        {
        Ok(pos) => Some(self.attrs[pos].attr_value),
        Err(_) => None,
        }
    }

    fn set_constraint(&mut self, attr_type: physio::udi_dma_constraints_attr_t, attr_value: u32) {
        match self.attrs.binary_search_by_key(&attr_type, |v| v.attr_type)
        {
        Ok(pos) => self.attrs[pos].attr_value = attr_value,
        Err(pos) => self.attrs.insert(pos, physio::udi_dma_constraints_attr_spec_t { attr_type, attr_value })
        }
    }
    fn clear_constraint(&mut self, attr_type: physio::udi_dma_constraints_attr_t) {
        match self.attrs.binary_search_by_key(&attr_type, |v| v.attr_type)
        {
        Ok(pos) => { self.attrs.remove(pos); },
        Err(_) => {},
        }
    }
}

#[no_mangle]
unsafe extern "C" fn udi_dma_constraints_attr_set(
    callback: physio::udi_dma_constraints_attr_set_call_t,
    gcb: *mut udi_cb_t,
    mut src_constraints: physio::udi_dma_constraints_t,
    attr_list: *const physio::udi_dma_constraints_attr_spec_t,
    list_length: udi_ubit16_t,
    flags: udi_ubit8_t
    )
{
    if flags & physio::UDI_DMA_CONSTRAINTS_COPY != 0 {
        if !src_constraints.is_null() {
            let sc = &*(src_constraints as *const ConstaintsReal);
            let sc = Box::new( sc.clone() );
            src_constraints = Box::into_raw(sc) as physio::udi_dma_constraints_t;
        }
    }
    let attrs = ::core::slice::from_raw_parts(attr_list, list_length as _);
    let status = if attrs.is_empty() {
            UDI_OK
        }
        else {
            let sc = if src_constraints.is_null() {
                let sc = Box::leak(Box::new(ConstaintsReal::default()));
                src_constraints = sc as *mut _ as _;
                sc
            }
            else {
                &mut *(src_constraints as *mut ConstaintsReal)
            };
            for a in attrs {
                sc.set_constraint(a.attr_type, a.attr_value);
            }
            UDI_OK
        };
        crate::async_call(gcb, move |gcb| callback(gcb, src_constraints, status as _))
}
#[no_mangle]
unsafe extern "C" fn udi_dma_constraints_attr_reset(
    constraints: physio::udi_dma_constraints_t,
    attr_type: physio::udi_dma_constraints_attr_t
    )
{
    let sc = &mut *(constraints as *mut ConstaintsReal);
    sc.clear_constraint(attr_type);
}
#[no_mangle]
pub unsafe extern "C" fn udi_dma_constraints_free(constraints: physio::udi_dma_constraints_t)
{
    if constraints != physio::UDI_NULL_DMA_CONSTRAINTS {
        drop(Box::from_raw(constraints as *mut ConstaintsReal));
    }
}
