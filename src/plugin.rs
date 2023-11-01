#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]
cfg_if::cfg_if! {
    if #[cfg(all(target_os = "linux", target_arch = "x86_64"))] {
        include!("bindings_linux_x86_64.rs");
    } else {
        std::compile_error!("pre-generated bindings are not avaliable for your target");
    }
}

use libc::{c_char, c_int, c_uint};
use libloading::Library;
use std::{error::Error, path::Path};

use std::sync::Once;

static INIT: Once = Once::new();
static mut INSTANCE: Option<AardvarkApi> = None;

type AaI2cReadFn = extern "C" fn(
    aardvark: Aardvark,
    slave_addr: u16_,
    flags: AardvarkI2cFlags,
    num_bytes: u16_,
    data_in: *mut u08,
) -> c_int;

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
type AaTargetPowerFn = extern "C" fn(aardvark: Aardvark, power_mask: u08) -> c_int;

type AaConfigureFn = extern "C" fn(aardvark: Aardvark, config: AardvarkConfig) -> c_int;

type AaI2cSlaveSetResponseFn =
    extern "C" fn(aardvark: Aardvark, num_bytes: u08, data_out: *const u08) -> c_int;

type AaI2cSlaveDisable = extern "C" fn(aardvark: Aardvark) -> c_int;
type AaAsyncPollFn = extern "C" fn(aardvark: Aardvark, timeout: c_int) -> c_int;
type AaI2cSlaveReadFn =
    extern "C" fn(aardvark: Aardvark, addr: *mut u08, num_bytes: u16_, data_in: *mut u08) -> c_int;

type AaFindDevicesExtFn = extern "C" fn(
    num_devices: ::std::os::raw::c_int,
    devices: *mut u16_,
    num_ids: ::std::os::raw::c_int,
    unique_ids: *mut u32_,
) -> c_int;

pub const AARDVARK_LIB: &str = "dynamic-lib/aardvark.so";

#[derive(Default, Clone)]
pub struct AardvarkApi {
    // Each field should be an Option<T> to allow for lazy initialization
    aa_i2c_read: Option<AaI2cReadFn>,
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
    aa_find_devices_ext: Option<AaFindDevicesExtFn>,
    aa_configure: Option<AaConfigureFn>,
    aa_target_power: Option<AaTargetPowerFn>,
    aa_i2c_slave_set_response: Option<AaI2cSlaveSetResponseFn>,
    aa_i2c_slave_disable: Option<AaI2cSlaveDisable>,
    aa_async_poll: Option<AaAsyncPollFn>,
    aa_i2c_slave_read: Option<AaI2cSlaveReadFn>,
}

impl AardvarkApi {
    /// load library and find pointers. If any of the pointers are null, return an error
    /// Otherwise, return the struct.
    /// # Safety
    pub unsafe fn try_load(lib_path: &str) -> Result<Self, Box<dyn Error>> {
        if !Path::new(lib_path).exists() {
            return Err(format!("{} does not exist", lib_path).into());
        }
        let library = Library::new(lib_path)?;

        let aa_i2c_read = library.get(b"c_aa_i2c_read\0")?;
        let aa_i2c_read_ext = library.get(b"c_aa_i2c_read_ext\0")?;
        let aa_i2c_write = library.get(b"c_aa_i2c_write\0")?;
        let aa_i2c_write_ext = library.get(b"c_aa_i2c_write_ext\0")?;
        let aa_i2c_write_read = library.get(b"c_aa_i2c_write_read\0")?;
        let aa_i2c_slave_enable = library.get(b"c_aa_i2c_slave_enable\0")?;
        let aa_close = library.get(b"c_aa_close\0")?;
        let aa_port = library.get(b"c_aa_port\0")?;
        let aa_features = library.get(b"c_aa_features\0")?;
        let aa_unique_id = library.get(b"c_aa_unique_id\0")?;
        let aa_status_string = library.get(b"c_aa_status_string\0")?;
        let aa_log = library.get(b"c_aa_log\0")?;
        let aa_gpio_direction = unsafe { library.get(b"c_aa_gpio_direction\0") }?;
        let aa_gpio_pullup = unsafe { library.get(b"c_aa_gpio_pullup\0") }?;
        let aa_gpio_get = library.get(b"c_aa_gpio_get\0")?;
        let aa_gpio_set = library.get(b"c_aa_gpio_set\0")?;
        let aa_gpio_change = library.get(b"c_aa_gpio_change\0")?;
        let aa_find_devices = library.get(b"c_aa_find_devices\0")?;
        let aa_find_devices_ext = library.get(b"c_aa_find_devices_ext\0")?;
        let aa_configure = library.get(b"c_aa_configure\0")?;
        let aa_target_power = library.get(b"c_aa_target_power")?;
        let aa_i2c_slave_set_response = library.get(b"c_aa_i2c_slave_set_response")?;
        let aa_i2c_slave_disable = library.get(b"c_aa_i2c_slave_disable")?;
        let aa_async_poll = library.get(b"c_aa_async_poll")?;
        let aa_i2c_slave_read = library.get(b"c_aa_i2c_slave_read")?;
        Ok(Self {
            aa_i2c_read: Some(*aa_i2c_read),
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
            aa_find_devices_ext: Some(*aa_find_devices_ext),
            aa_configure: Some(*aa_configure),
            aa_target_power: Some(*aa_target_power),
            aa_i2c_slave_set_response: Some(*aa_i2c_slave_set_response),
            aa_i2c_slave_disable: Some(*aa_i2c_slave_disable),
            aa_async_poll: Some(*aa_async_poll),
            aa_i2c_slave_read: Some(*aa_i2c_slave_read),
        })
    }

    /// This new function invokes try_load and returns an Option with the instance.
    pub fn new() -> Option<Self> {
        INIT.call_once(|| unsafe {
            INSTANCE = Self::try_load(AARDVARK_LIB).ok();
        });

        unsafe { INSTANCE.clone() }
    }

    pub fn aa_unique_id(&self, aardvark: Aardvark) -> u32_ {
        self.aa_unique_id.unwrap()(aardvark)
    }

    pub fn aa_i2c_read(
        &self,
        aardvark: Aardvark,
        slave_addr: u16_,
        flags: AardvarkI2cFlags,
        num_bytes: u16_,
        data_in: *mut u08,
    ) -> c_int {
        self.aa_i2c_read.unwrap()(aardvark, slave_addr, flags, num_bytes, data_in)
    }

    pub fn aa_i2c_read_ext(
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

    pub fn aa_i2c_write_ext(
        &self,
        aardvark: Aardvark,
        slave_addr: u16_,
        flags: AardvarkI2cFlags,
        num_bytes: u16_,
        data_out: *const u08,
        num_written: *mut u16_,
    ) -> c_int {
        self.aa_i2c_write_ext.unwrap()(
            aardvark,
            slave_addr,
            flags,
            num_bytes,
            data_out,
            num_written,
        )
    }

    pub fn aa_i2c_write_read(
        &self,
        aardvark: Aardvark,
        slave_addr: u16_,
        flags: AardvarkI2cFlags,
        out_num_bytes: u16_,
        out_data: *const u08,
        num_written: *mut u16_,
        in_num_bytes: u16_,
        in_data: *mut u08,
        num_read: *mut u16_,
    ) -> c_int {
        self.aa_i2c_write_read.unwrap()(
            aardvark,
            slave_addr,
            flags,
            out_num_bytes,
            out_data,
            num_written,
            in_num_bytes,
            in_data,
            num_read,
        )
    }

    pub fn aa_open(&self, port: c_int) -> Aardvark {
        self.aa_port.unwrap()(port)
    }
    pub fn aa_find_devices(&self, num_devices: c_int, devices: *mut u16_) -> c_int {
        self.aa_find_devices.unwrap()(num_devices, devices)
    }
    pub fn aa_find_devices_ext(
        &self,
        num_devices: ::std::os::raw::c_int,
        devices: *mut u16_,
        num_ids: ::std::os::raw::c_int,
        unique_ids: *mut u32_,
    ) -> c_int {
        self.aa_find_devices_ext.unwrap()(num_devices, devices, num_ids, unique_ids)
    }

    pub fn aa_configure(&self, aardvark: Aardvark, config: AardvarkConfig) -> c_int {
        self.aa_configure.unwrap()(aardvark, config)
    }
    pub fn aa_target_power(&self, aardvark: Aardvark, power_mask: u08) -> c_int {
        self.aa_target_power.unwrap()(aardvark, power_mask)
    }
    pub fn aa_close(&self, aardvark: Aardvark) -> c_int {
        self.aa_close.unwrap()(aardvark)
    }
    pub fn aa_i2c_slave_set_response(
        &self,
        aardvark: Aardvark,
        num_bytes: u08,
        data_out: *const u08,
    ) -> c_int {
        self.aa_i2c_slave_set_response.unwrap()(aardvark, num_bytes, data_out)
    }
    pub fn aa_i2c_slave_enable(
        &self,
        aardvark: Aardvark,
        addr: u08,
        maxTxBytes: u16_,
        maxRxBytes: u16_,
    ) -> c_int {
        self.aa_i2c_slave_enable.unwrap()(aardvark, addr, maxTxBytes, maxRxBytes)
    }
    pub fn aa_i2c_slave_disable(&self, aardvark: Aardvark) {
        self.aa_i2c_slave_disable.unwrap()(aardvark);
    }
    pub fn aa_i2c_slave_read(
        &self,
        aardvark: Aardvark,
        addr: *mut u08,
        num_bytes: u16_,
        data_in: *mut u08,
    ) -> c_int {
        self.aa_i2c_slave_read.unwrap()(aardvark, addr, num_bytes, data_in)
    }

    pub fn aa_async_poll(&self, aardvark: Aardvark, timeout: c_int) -> c_int {
        self.aa_async_poll.unwrap()(aardvark, timeout)
    }

    pub fn aa_status_string(&self, status: c_int) -> *const c_char {
        self.aa_status_string.unwrap()(status)
    }
    pub fn aa_gpio_direction(&self, aardvark: Aardvark, direction_mask: u08) -> c_int {
        self.aa_gpio_direction.unwrap()(aardvark, direction_mask)
    }
    pub fn aa_gpio_pullup(&self, aardvark: Aardvark, pullup_mask: u08) -> c_int {
        self.aa_gpio_pullup.unwrap()(aardvark, pullup_mask)
    }
    pub fn aa_gpio_change(&self, aardvark: Aardvark, timeout: u16_) -> c_int {
        self.aa_gpio_change.unwrap()(aardvark, timeout)
    }
    pub fn aa_features(&self, aardvark: Aardvark) -> c_int {
        self.aa_features.unwrap()(aardvark)
    }
    pub fn aa_log(&self, aardvark: Aardvark, level: c_int, handle: c_int) -> c_int {
        self.aa_log.unwrap()(aardvark, level, handle)
    }
    pub fn aa_gpio_get(&self, aardvark: Aardvark) -> c_int {
        self.aa_gpio_get.unwrap()(aardvark)
    }
    pub fn aa_gpio_set(&self, aardvark: Aardvark, value: u08) -> c_int {
        self.aa_gpio_set.unwrap()(aardvark, value)
    }
}

// Generate unit tests now:
#[test]
fn test_aardvark_api_load() {
    match unsafe { AardvarkApi::try_load(AARDVARK_LIB) } {
        Ok(_) => println!("Aardvark API loaded successfully"),
        Err(e) => println!("Aardvark API failed to load: {}", e),
    }
}
#[test]
fn test_aardvark_api_new() {
    assert!(AardvarkApi::new().is_some());
}
