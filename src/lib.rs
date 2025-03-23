use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

pub mod config;
pub mod fan;
pub mod tui;
pub mod constants;
pub use constants::*;

pub fn init_loop() {
    let in_band = false;
    if in_band {
        println!("Running with in-band mode");
    } else {
        println!("Running with out-band mode");
    }

    let config_path = format!("{}/HR650X.yaml", std::env::current_dir().unwrap().display());
    let mut file = File::open(config_path).expect("Failed to open config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read config file");

    let config: config::Config = serde_yaml::from_str(&contents).expect("Failed to parse config file");

    let ipmi_tool_cmd = if in_band {
        "ipmitool".to_string()
    } else {
        format!(
            "ipmitool -I lanplus -H {} -U {} -P '{}'",
            config.ipmi.host, config.ipmi.username, config.ipmi.password
        )
    };

    let mut cpu2_fan_speed_set = false;

    loop {
        if let Some((temp, cpu_num)) = fan::get_temperature_and_cpu_num(&ipmi_tool_cmd) {
            let speed = fan::get_fan_speed(temp, &config.fan_speeds);
            if fan::set_fan_speed(speed, &ipmi_tool_cmd, cpu_num, &mut cpu2_fan_speed_set) {
                println!("Set fan speed to {}% for CPU temperature {}°C", speed, temp);
            }
        }
        thread::sleep(Duration::from_secs(10));
    }
}
