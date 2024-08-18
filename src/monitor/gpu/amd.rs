//! Reads live GPU data from the Linux kernel.

use std::{fs::read_dir, fs::read_to_string, process::exit};

pub struct Gpu {
    temp_sensor: String,
    usage_file: String,
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            temp_sensor: find_temp_sensor(),
            usage_file: find_card(),
        }
    }

    /// Reads the value of the GPU temperature sensor and calculates it to be `˚C` or `˚F`.
    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        // Read sensor data
        let data = read_to_string(&self.temp_sensor).expect("GPU temperature cannot be read!");

        // Calculate temperature
        let mut temp = data.trim_end().parse::<u32>().unwrap();
        if fahrenheit {
            temp = temp * 9 / 5 + 32000
        }

        (temp as f32 / 1000.0).round() as u8
    }

    /// Reads the value of the GPU usage in percentage.
    pub fn get_usage(&self) -> u8 {
        let data = read_to_string(&self.usage_file).expect("GPU usage cannot be read!");

        data.trim_end().parse::<u8>().unwrap()
    }
}

/// Looks for the appropriate CPU temperature sensor datastream in the hwmon folder.
fn find_temp_sensor() -> String {
    let mut i = 0;
    loop {
        match read_to_string(format!("/sys/class/hwmon/hwmon{i}/name")) {
            Ok(data) => {
                let hwname = data.trim_end();
                if hwname == "amdgpu" {
                    return format!("/sys/class/hwmon/hwmon{i}/temp1_input");
                }
            }
            Err(_) => {
                println!("AMD GPU temperature sensor not found!");
                exit(1);
            }
        }
        i += 1;
    }
}

/// Looks for the PCI device folder of the AMD GPU.
fn find_card() -> String {
    match read_dir("/sys/bus/pci/devices") {
        Ok(devices) => {
            for device in devices {
                let path = device.unwrap().path().to_str().unwrap().to_owned();
                match read_to_string(format!("{path}/uevent")) {
                    Ok(data) => {
                        let driver = data.lines().next().unwrap();
                        if driver.ends_with("amdgpu") {
                            return format!("{path}/gpu_busy_percent");
                        }
                    }
                    Err(_) => (),
                }
            }
        }
        Err(_) => (),
    }
    println!("AMD PCI data not found!");
    exit(1);
}