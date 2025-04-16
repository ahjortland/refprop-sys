use std::ffi::c_int;

use crate::{bindings, utils::acquire_lock, RefpropError, RefpropFunctionLibrary};

impl RefpropFunctionLibrary {
    pub fn pure_fld(icomp: usize) -> Result<(), RefpropError> {
        // Acquire the mutex lock to ensure exclusive access
        let _guard = acquire_lock()?;

        let mut icomp_mut = icomp as i32 as c_int;
        let icomp_ptr = &mut icomp_mut as *mut c_int;
        // Call PUREFLDdll within an unsafe block
        unsafe {
            bindings::PUREFLDdll(icomp_ptr);
        }

        Ok(())
    }
}
