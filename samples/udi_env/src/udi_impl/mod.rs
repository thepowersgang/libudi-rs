


pub mod cb;
pub mod imc;
pub mod pio;
pub mod buf;

macro_rules! dispatch_call {
    ( $($vis:vis fn $name:ident(cb: *mut $cb_ty:ty $(, $a_name:ident: $a_ty:ty)*) => $ops_ty:ty : $ops_name:ident;)+) => {
        $(
        #[no_mangle]
        $vis unsafe extern "C" fn $name(cb: *mut $cb_ty $(, $a_name : $a_ty)*) {
            let ops = crate::channels::prepare_cb_for_call::<$ops_ty>(&mut (*cb).gcb);
            (ops.$ops_name)(cb $(, $a_name)*);
        }
    )+
    };
}

pub mod meta_bus;
pub mod meta_intr;
pub mod meta_mgmt;
pub mod meta_nic;