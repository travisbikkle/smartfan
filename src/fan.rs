use crate::config;
use std::process::Command;

pub fn get_temperature_and_cpu_num(ipmi_tool_cmd: &str) -> Option<(f64, u8)> {
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

pub fn extract_temperature(temp_str: &str) -> Option<f64> {
    temp_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
}

pub fn set_fan_speed(speed: u8, ipmi_tool_cmd: &str, cpu_num: u8, cpu2_fan_speed_set: &mut bool) -> bool {
    println!(
        "CPU number is {}, CPU 2 fan turned off? {}",
        cpu_num, cpu2_fan_speed_set
    );

    let mut delimiter = if cfg!(target_os = "windows") {
        "&"
    } else {
        ";"
    };

    let mut cmd = if cpu_num == 1 {
        let mut cmd = format!(
            "{} raw 0x2e 0x30 00 01 {}{} {} raw 0x2e 0x30 00 02 {}{} {} raw 0x2e 0x30 00 03 {}",
            ipmi_tool_cmd, speed, delimiter, ipmi_tool_cmd, speed, delimiter, ipmi_tool_cmd, speed
        );

        if !*cpu2_fan_speed_set {
            cmd.push_str(&format!(
                "{} {} raw 0x2e 0x30 00 04 02{} {} raw 0x2e 0x30 00 05 02{} {} raw 0x2e 0x30 00 06 02",
                delimiter, ipmi_tool_cmd, delimiter, ipmi_tool_cmd, delimiter, ipmi_tool_cmd
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

pub fn get_fan_speed(temp: f64, fan_speeds: &[config::FanSpeed]) -> u8 {
    for fan_speed in fan_speeds {
        if fan_speed.temp_range[0] <= temp && temp < fan_speed.temp_range[1] {
            return fan_speed.speed;
        }
    }
    100
}