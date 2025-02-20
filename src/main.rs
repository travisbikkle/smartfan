use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{Duration, SystemTime};
use std::thread;
use std::fs::File;
use std::io::Read;
use chrono::Local;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    ipmi: IpmiHostInfo,
    fan_speeds: Vec<FanSpeed>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IpmiHostInfo {
    host: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FanSpeed {
    temp_range: [f64; 2],
    speed: u8,
}

fn get_timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn get_temperature_and_cpu_num(ipmi_tool_cmd: &str) -> Option<(f64, u8)> {
    let cmd = format!("{} sensor", ipmi_tool_cmd);
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", &cmd])
            .output()
            .expect("Failed to execute command")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("Failed to execute command")
    };

    if !output.status.success() {
        eprintln!(
            "Error executing command: {}. Error: {}",
            cmd,
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines = output_str.lines();
    let mut temperatures = Vec::new();
    let mut cpu_num = 2;

    // CPU1_Temp        | 34.000     | degrees C  | ok    | na        | na        | na        | 93.000    | 100.000   | 105.000   
    // CPU2_Temp        | 0.000      | degrees C  | ok    | na        | na        | na        | 100.000   | 102.000   | 104.000   
    // CPU1_VR_Temp     | 30.000     | degrees C  | ok    | na        | na        | na        | 112.000   | 123.000   | 133.000   
    // CPU2_VR_Temp     | 15.000     | degrees C  | ok    | na        | na        | na        | 112.000   | 123.000   | 133.000   
    for line in lines {
        if line.is_empty() || !line.contains("CPU") || !line.contains("Temp") {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 2 {
            eprintln!("Unexpected line format: {}", line);
            continue;
        }

        let temp_str = parts[1].trim();
        if temp_str == "na" {
            println!("The system is off, temperature is na");
            temperatures.push(0.0);
            continue;
        }

        if let Some(temp) = extract_temperature(temp_str) {
            temperatures.push(temp);
            if line.contains("CPU2_Temp") && temp == 0.0 {
                cpu_num = 1;
            }
        }
    }

    if temperatures.is_empty() {
        eprintln!("No temperature data found.");
        return None;
    }

    let max_temp = temperatures
        .into_iter()
        .fold(0.0, |acc, temp| if temp > acc { temp } else { acc });

    Some((max_temp, cpu_num))
}

fn extract_temperature(temp_str: &str) -> Option<f64> {
    temp_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
}

fn set_fan_speed(speed: u8, ipmi_tool_cmd: &str, cpu_num: u8, cpu2_fan_speed_set: &mut bool) -> bool {
    println!(
        "CPU number is {}, CPU 2 fan turned off? {}",
        cpu_num, cpu2_fan_speed_set
    );

    let mut cmd = if cpu_num == 1 {
        let mut cmd = format!(
            "{} raw 0x2e 0x30 00 01 {}; {} raw 0x2e 0x30 00 02 {}; {} raw 0x2e 0x30 00 03 {}",
            ipmi_tool_cmd, speed, ipmi_tool_cmd, speed, ipmi_tool_cmd, speed
        );

        if !*cpu2_fan_speed_set {
            cmd.push_str(&format!(
                "; {} raw 0x2e 0x30 00 04 02; {} raw 0x2e 0x30 00 05 02; {} raw 0x2e 0x30 00 06 02",
                ipmi_tool_cmd, ipmi_tool_cmd, ipmi_tool_cmd
            ));
            *cpu2_fan_speed_set = true;
        }

        cmd
    } else {
        format!("{} raw 0x2e 0x30 00 00 {:02x}", ipmi_tool_cmd, speed)
    };

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", &cmd])
            .output()
            .expect("Failed to execute command")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("Failed to execute command")
    };

    if !output.status.success() {
        eprintln!(
            "Error executing command: {}. Error: {}",
            cmd,
            String::from_utf8_lossy(&output.stderr)
        );
        return false;
    }

    true
}

fn get_fan_speed(temp: f64, fan_speeds: &[FanSpeed]) -> u8 {
    for fan_speed in fan_speeds {
        if fan_speed.temp_range[0] <= temp && temp < fan_speed.temp_range[1] {
            return fan_speed.speed;
        }
    }
    100
}

fn main() {
    let in_band = std::env::args().any(|arg| arg == "in-band");
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

    let config: Config = serde_yaml::from_str(&contents).expect("Failed to parse config file");

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
        println!("{}", get_timestamp());
        if let Some((temp, cpu_num)) = get_temperature_and_cpu_num(&ipmi_tool_cmd) {
            let speed = get_fan_speed(temp, &config.fan_speeds);
            if set_fan_speed(speed, &ipmi_tool_cmd, cpu_num, &mut cpu2_fan_speed_set) {
                println!("Set fan speed to {}% for CPU temperature {}°C", speed, temp);
            }
        }
        thread::sleep(Duration::from_secs(10));
    }
}