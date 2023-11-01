use std::{env, ffi::CStr};

use aardvark_ffi as aardvark;

use aardvark::plugin::{
    Aardvark, AardvarkApi, AardvarkConfig_AA_CONFIG_SPI_I2C, AA_ASYNC_I2C_READ, AA_ASYNC_NO_DATA,
    AA_TARGET_POWER_NONE,
};

const SLAVE_RESP_SIZE: usize = 26;
const BUFFER_SIZE: usize = 65535;

#[derive(Default)]
struct Parameters(i32, u8, i32);

fn watch_i2c_port(api: &AardvarkApi, handle: Aardvark, timeout_ms: i32) {
    println!("Watching slave I2C data...\n");

    // Wait for data on bus
    let result = api.aa_async_poll(handle, timeout_ms);
    if result == AA_ASYNC_NO_DATA.try_into().unwrap() {
        println!("No data available.\n");
        return;
    }

    println!("\n");

    let mut data_in = [0u8; BUFFER_SIZE];
    let mut addr = [0u8; 1];

    loop {
        // Read the I2C message.
        // This function has an internal timeout (see datasheet), though
        // since we have already checked for data using aa_async_poll,
        // the timeout should never be exercised.
        if result == AA_ASYNC_I2C_READ.try_into().unwrap() {
            // Get data written by master
            let num_bytes = api.aa_i2c_slave_read(
                handle,
                addr.as_mut_ptr(),
                BUFFER_SIZE.try_into().unwrap(),
                data_in.as_mut_ptr(),
            );
            if num_bytes < 0 {
                let char_ptr = api.aa_status_string(num_bytes);
                let c_str = unsafe { CStr::from_ptr(char_ptr) };
                println!("error: {}", c_str.to_str().unwrap());
                break;
            }
        }
    }
}

pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("usage: aai2c_slave PORT SLAVE_ADDR TIMEOUT_MS\n");
        println!("  SLAVE_ADDR is the slave address for this device\n");
        println!("\n");
        println!("  The timeout value specifies the time to\n");
        println!("  block until the first packet is received.\n");
        println!("  If the timeout is -1, the program will\n");
        println!("  block indefinitely.\n");
        return;
    }

    let params = Parameters(
        args[1].parse().unwrap(),
        args[2].parse().unwrap(),
        args[3].parse().unwrap(),
    );
    let Parameters(port, addr, timeout_ms) = params;

    // Open the device
    let api = unsafe { AardvarkApi::try_load("./dynamic-lib/libaardvark.so").unwrap() };

    let handle = api.aa_open(port);
    if handle <= 0 {
        println!("Unable to open Aardvark device on port {}", params.0);
        println!("Error code {}", handle);
        return;
    }
    // Ensure that the I2C subsystem is enabled
    api.aa_configure(handle, AardvarkConfig_AA_CONFIG_SPI_I2C);

    // Disable the Aardvark adapter's power pins.
    // This command is only effective on v2.0 hardware or greater.
    // The power pins on the v1.02 hardware are not enabled by default.
    api.aa_target_power(handle, AA_TARGET_POWER_NONE.try_into().unwrap());

    let alphabet = b'A'..=b'Z';

    let slave_resp: Vec<u8> = alphabet.collect();

    // Set the slave response; this won't be used unless the master
    // reads bytes from the slave.
    api.aa_i2c_slave_set_response(handle, SLAVE_RESP_SIZE as u8, slave_resp.as_ptr());

    api.aa_i2c_slave_enable(handle, addr, 0, 0);

    watch_i2c_port(&api, handle, timeout_ms);

    api.aa_i2c_slave_disable(handle);

    api.aa_close(handle);
}
