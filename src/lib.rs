use libc::{c_char, c_int, c_uchar, c_uint, c_ushort};
use libloading::{Library, Symbol};
use std::{error::Error, path::Path, sync::Arc};

use std::sync::Once;

static INIT: Once = Once::new();
static mut INSTANCE: Option<AardvarkApi> = None;

// Define custom types as aliases for better readability
pub type Aardvark = c_int;
pub type AardvarkI2cFlags = c_ushort;
pub type u08 = c_uchar;
pub type u16_ = c_ushort;

// Type for aa_i2c_read_ext function pointer
type AaI2cReadExtFn =
    extern "C" fn(Aardvark, u16_, AardvarkI2cFlags, u16_, *mut u08, *mut u16_) -> c_int;

// Type for aa_i2c_write function pointer
type AaI2cWriteFn = extern "C" fn(Aardvark, u16_, AardvarkI2cFlags, u16_, *const u08) -> c_int;

// Type for aa_i2c_write_ext function pointer
type AaI2cWriteExtFn =
    extern "C" fn(Aardvark, u16_, AardvarkI2cFlags, u16_, *const u08, *mut u16_) -> c_int;

// Type for aa_i2c_write_read function pointer
type AaI2cWriteReadFn = extern "C" fn(
    Aardvark,
    u16_,
    AardvarkI2cFlags,
    u16_,
    *const u08,
    *mut u16_,
    u16_,
    *mut u08,
    *mut u16_,
) -> c_int;

// Type for aa_i2c_slave_enable function pointer
type AaI2cSlaveEnableFn = extern "C" fn(Aardvark, u08, u16_, u16_) -> c_int;

type AaCloseFn = extern "C" fn(aardvark: Aardvark) -> c_int;
type AaPortFn = extern "C" fn(aardvark: Aardvark) -> c_int;
type AaFeaturesFn = extern "C" fn(aardvark: Aardvark) -> c_int;
type AaUniqueIdFn = extern "C" fn(aardvark: Aardvark) -> c_uint;
type AaStatusStringFn = extern "C" fn(status: c_int) -> *const c_char;
type AaLogFn = extern "C" fn(aardvark: Aardvark, level: c_int, handle: c_int) -> c_int;

type AaGpioDirectionFn = extern "C" fn(aardvark: Aardvark, direction_mask: u08) -> c_int;
type AaGpioPullupFn = extern "C" fn(aardvark: Aardvark, pullup_mask: u08) -> c_int;
type AaGpioGetFn = extern "C" fn(aardvark: Aardvark) -> c_int;
type AaGpioSetFn = extern "C" fn(aardvark: Aardvark, value: u08) -> c_int;
type AaGpioChangeFn = extern "C" fn(aardvark: Aardvark, timeout: u16_) -> c_int;
type AaFindDevicesFn = extern "C" fn(c_int, *mut u16_) -> c_int;

const AARDVARK_LIB: &str = "dynamic-lib/aardvark.so";

#[derive(Default, Clone)]
pub struct AardvarkApi {
    // Each field should be an Option<T> to allow for lazy initialization
    aa_i2c_read_ext: Option<AaI2cReadExtFn>,
    aa_i2c_write: Option<AaI2cWriteFn>,
    aa_i2c_write_ext: Option<AaI2cWriteExtFn>,
    aa_i2c_write_read: Option<AaI2cWriteReadFn>,
    aa_i2c_slave_enable: Option<AaI2cSlaveEnableFn>,
    aa_close: Option<AaCloseFn>,
    aa_port: Option<AaPortFn>,
    aa_features: Option<AaFeaturesFn>,
    aa_unique_id: Option<AaUniqueIdFn>,
    aa_status_string: Option<AaStatusStringFn>,
    aa_log: Option<AaLogFn>,
    aa_gpio_direction: Option<AaGpioDirectionFn>,
    aa_gpio_pullup: Option<AaGpioPullupFn>,
    aa_gpio_get: Option<AaGpioGetFn>,
    aa_gpio_set: Option<AaGpioSetFn>,
    aa_gpio_change: Option<AaGpioChangeFn>,
    aa_find_devices: Option<AaFindDevicesFn>,
}

impl AardvarkApi {
    /// load library and find pointers. If any of the pointers are null, return an error
    /// Otherwise, return the struct.
    pub fn try_load(lib_path: &str) -> Result<Self, Box<dyn Error>> {
        if !Path::new(lib_path).exists() {
            return Err(format!("{} does not exist", lib_path).into());
        }
        let library = unsafe { Library::new(AARDVARK_LIB) }?;

        let aa_i2c_read_ext = unsafe { library.get(b"c_aa_i2c_read_ext\0") }?;
        let aa_i2c_write = unsafe { library.get(b"c_aa_i2c_write\0") }?;
        let aa_i2c_write_ext = unsafe { library.get(b"c_aa_i2c_write_ext\0") }?;
        let aa_i2c_write_read = unsafe { library.get(b"c_aa_i2c_write_read\0") }?;
        let aa_i2c_slave_enable = unsafe { library.get(b"c_aa_i2c_slave_enable\0") }?;
        let aa_close = unsafe { library.get(b"c_aa_close\0") }?;
        let aa_port = unsafe { library.get(b"c_aa_port\0") }?;
        let aa_features = unsafe { library.get(b"c_aa_features\0") }?;
        let aa_unique_id = unsafe { library.get(b"c_aa_unique_id\0") }?;
        let aa_status_string = unsafe { library.get(b"c_aa_status_string\0") }?;
        let aa_log = unsafe { library.get(b"c_aa_log\0") }?;
        let aa_gpio_direction = unsafe { library.get(b"c_aa_gpio_direction\0") }?;
        let aa_gpio_pullup = unsafe { library.get(b"c_aa_gpio_pullup\0") }?;
        let aa_gpio_get = unsafe { library.get(b"c_aa_gpio_get\0") }?;
        let aa_gpio_set = unsafe { library.get(b"c_aa_gpio_set\0") }?;
        let aa_gpio_change = unsafe { library.get(b"c_aa_gpio_change\0") }?;
        let aa_find_devices = unsafe { library.get(b"c_aa_find_devices\0") }?;

        Ok(Self {
            aa_i2c_read_ext: Some(*aa_i2c_read_ext),
            aa_i2c_write: Some(*aa_i2c_write),
            aa_i2c_write_ext: Some(*aa_i2c_write_ext),
            aa_i2c_write_read: Some(*aa_i2c_write_read),
            aa_i2c_slave_enable: Some(*aa_i2c_slave_enable),
            aa_close: Some(*aa_close),
            aa_port: Some(*aa_port),
            aa_features: Some(*aa_features),
            aa_unique_id: Some(*aa_unique_id),
            aa_status_string: Some(*aa_status_string),
            aa_log: Some(*aa_log),
            aa_gpio_direction: Some(*aa_gpio_direction),
            aa_gpio_pullup: Some(*aa_gpio_pullup),
            aa_gpio_get: Some(*aa_gpio_get),
            aa_gpio_set: Some(*aa_gpio_set),
            aa_gpio_change: Some(*aa_gpio_change),
            aa_find_devices: Some(*aa_find_devices),
        })
    }

    /// This new function invokes try_load and returns an Option with the instance.
    pub fn new() -> Option<Self> {
        INIT.call_once(|| unsafe {
            INSTANCE = Self::try_load(AARDVARK_LIB).ok();
        });

        unsafe { INSTANCE.clone() }
    }
    pub fn aa_i2c_read(
        &self,
        aardvark: Aardvark,
        slave_addr: u16_,
        flags: AardvarkI2cFlags,
        num_bytes: u16_,
        data_in: *mut u08,
        num_read: *mut u16_,
    ) -> c_int {
        self.aa_i2c_read_ext.unwrap()(aardvark, slave_addr, flags, num_bytes, data_in, num_read)
    }
    pub fn aa_i2c_write(
        &self,
        aardvark: Aardvark,
        slave_addr: u16_,
        flags: AardvarkI2cFlags,
        num_bytes: u16_,
        data_out: *const u08,
    ) -> c_int {
        self.aa_i2c_write.unwrap()(aardvark, slave_addr, flags, num_bytes, data_out)
    }
    pub fn aa_open(&self, port: c_int) -> Aardvark {
        self.aa_port.unwrap()(port)
    }
    pub fn aa_find_devices(&self, num_devices: c_int, devices: *mut u16_) -> c_int {
        self.aa_find_devices.unwrap()(num_devices, devices)
    }
}

/// Aardvark API wrapper
/// This struct is thread-safe and can be shared across threads
/// It is also Send and Sync
/// It is also lazy-initialized
/// It is also cloneable
/// It is also cheap to clone
/// It is also cheap to create
/// It is also cheap to drop
struct AardvarkApiWrapper(Arc<AardvarkApi>);

// Implement Clone for AardvarkApiWrapper
impl Clone for AardvarkApiWrapper {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Generate unit tests now:
#[test]
fn test_aardvark_api_load() {
    match AardvarkApi::try_load(AARDVARK_LIB) {
        Ok(_) => println!("Aardvark API loaded successfully"),
        Err(e) => println!("Aardvark API failed to load: {}", e),
    }
}
#[test]
fn test_aardvark_api_new() {
    assert!(AardvarkApi::new().is_some());
}
