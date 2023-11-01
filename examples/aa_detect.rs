// Example using AA_find_devices
use aardvark_ffi as aardvark;

use aardvark::plugin::{AardvarkApi, AA_PORT_NOT_FREE};

//=========================================================================
// MAIN PROGRAM ENTRY POINT
//=========================================================================
fn main() {
    let api = unsafe { AardvarkApi::try_load("./dynamic-lib/libaardvark.so").unwrap() };
    let mut devices: [u16; 16] = [0; 16];
    let mut unique_ids: [u32; 16] = [0; 16];

    // Find all the attached devices
    let count = api.aa_find_devices_ext(
        devices.len() as i32,
        devices.as_mut_ptr(),
        unique_ids.len() as i32,
        unique_ids.as_mut_ptr(),
    );

    println!("{} device(s) found", count);
    let mut iter = unique_ids.iter();

    // Print the information on each device
    for mut device in devices {
        // Determine if the device is in-use
        let status = if (device & AA_PORT_NOT_FREE as u16) != 0 {
            device = device & !(AA_PORT_NOT_FREE as u16);
            "(in-use)"
        } else {
            ""
        };

        if let Some(unique_id) = iter.next() {
            let a = unique_id / 1000000_u32;
            let b = unique_id % 1000000_u32;

            println!("    port{} {} {} {}", device, status, a, b,);
        }
        // Display device port number, in-use status, and serial number
    }
}
