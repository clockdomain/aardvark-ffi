// Example using AA_find_devices
use aardvark_ffi::plugin::AardvarkApi;
fn main() {
    let api = unsafe { AardvarkApi::try_load("./dynamic-lib/libaardvark.so").unwrap() };
    let num_devices: i32 = 0;
    let mut devices: [u16; 16] = [0; 16];
    let count = api.aa_find_devices(num_devices, devices.as_mut_ptr());
    println!("Found {} devices", count);
}
