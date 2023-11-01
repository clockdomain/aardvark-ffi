use std::ffi::CStr;
use std::fmt;
use std::num::NonZeroI32;

use libc::c_int;
use plugin::AardvarkApi;

pub mod i2c;
pub mod plugin;

pub struct AardvarkError(std::num::NonZeroI32);

impl AardvarkError {
    pub const fn new_from_const(status: c_int) -> Self {
        match NonZeroI32::new(status as i32) {
            Some(val) => Self(val),
            None => panic!("AardvarkError cannot be 0"),
        }
    }
}

impl AardvarkError {
    fn get_aardvark_status_string(
        error: AardvarkError,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let api = unsafe { AardvarkApi::try_load(crate::plugin::AARDVARK_LIB) }?;

        let cstr = unsafe { CStr::from_ptr(api.aa_status_string(error.0.get() as c_int)) };

        match cstr.to_str() {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(From::from(e)),
        }
    }
}

impl fmt::Display for AardvarkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Aardvark error: {}", self.0.get().to_string().as_str())
    }
}

pub fn find_aardvark_devices() -> Vec<u16> {
    let api = unsafe { plugin::AardvarkApi::try_load("./dynamic-lib/libaardvark.so").unwrap() };
    let mut devices: [u16; 16] = [0; 16];
    let num_devices = api.aa_find_devices(devices.len() as i32, devices.as_mut_ptr());
    if num_devices < 0 {
        // Return empty vector if no devices are found
        return Vec::new();
    }

    let num_devices = (num_devices as usize).min(devices.len());
    // Truncate array to number of devices found or the size of the devices array
    devices[0..num_devices].to_vec()
}
