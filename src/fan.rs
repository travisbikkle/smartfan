use std::io;
use std::io::ErrorKind;
use crate::{config, Message};
use std::process::Command;
use tokio::sync::mpsc::Sender;
use crate::sensor::SensorResult;
use regex::Regex;

pub fn get_active_cpu_num(sensor_results: &Vec<SensorResult>) -> (usize, usize) {
    let mut num = 0;
    let mut max_num = 2;
    let cpu_re = Regex::new(r"(?i)(CPU|Processor|Proc)[_ ]?(\d+)").unwrap();
    sensor_results.iter()
        .filter(|&x|x.sensor_name.contains("Temp") && !x.sensor_name.contains("VR"))
        .for_each(|x| {
            if let Some(caps) = cpu_re.captures(&x.sensor_name) {
                if let Some(num_str) = caps.get(2) {
                    let current_id: usize = num_str.as_str().parse().unwrap();
                    max_num = max_num.max(current_id);
                    match x.value {
                        Some(v) => {
                            if v > 0.0 {
                                num = num.max(current_id);
                            }
                        }
                        None => ()
                    }
                }
            }
        });
    (num, max_num)
}

pub fn get_max_temperature(sensor_results: &Vec<SensorResult>) -> f64 {
    let mut max_temp = 0.0;
    sensor_results.iter()
        .filter(|&x| {x.sensor_name.contains("CPU") && x.sensor_name.contains("Temp")})
        .for_each(|x| {
            if x.value.is_some() && x.value.unwrap() > max_temp {
                max_temp = x.value.unwrap();
            }
        });
    max_temp
}

pub fn get_fans_speed(sensor_results: &Vec<SensorResult>) -> Vec<(String,f64)> {
    let mut fan_speeds = Vec::new();
    sensor_results.iter()
        .filter(|&x| x.sensor_name.contains("FAN") && x.sensor_name.contains("Speed"))
        .for_each(|x| {
            let mut speed: f64 = 0.0;
            if x.value.is_some() { speed = x.value.unwrap() };
            fan_speeds.push((x.sensor_name.clone().replace("FAN", "").replace("_Speed", ""), speed));
        });
    fan_speeds
}

pub fn get_all_sensor_data(ipmi_tool_cmd: &str) -> io::Result<Vec<SensorResult>> {
    let cmd = format!("{} sensor", ipmi_tool_cmd);
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", &cmd])
            .output()?
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()?
    };

    if !output.status.success() {
        let msg = format!(
            "Error executing command: {}. Error: {}",
            cmd,
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(io::Error::new(ErrorKind::Other, msg));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines = output_str.lines();

    // CPU1_Temp        | 34.000     | degrees C  | ok    | na        | na        | na        | 93.000    | 100.000   | 105.000
    // CPU2_Temp        | 0.000      | degrees C  | ok    | na        | na        | na        | 100.000   | 102.000   | 104.000
    // CPU1_VR_Temp     | 30.000     | degrees C  | ok    | na        | na        | na        | 112.000   | 123.000   | 133.000
    // CPU2_VR_Temp     | 15.000     | degrees C  | ok    | na        | na        | na        | 112.000   | 123.000   | 133.000
    let mut sensor_data = vec![];
    for line in lines {
        if let Ok(d) = SensorResult::from_line(line) {
            sensor_data.push(d);
        }
    }

    Ok(sensor_data)
}

pub fn extract_temperature(temp_str: &str) -> Option<f64> {
    temp_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
}

pub fn set_fan_speed(
    speed: u8,
    ipmi_tool_cmd: &str,
    cpu_num: usize,
    cpu2_fan_speed_set: &mut bool,
) -> io::Result<()> {
    let delimiter = if cfg!(target_os = "windows") {
        "&"
    } else {
        ";"
    };

    let cmd = if cpu_num == 1 {
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
        format!("{} raw 0x2e 0x30 00 00 {}", ipmi_tool_cmd, speed)
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
        return Err(io::Error::new(ErrorKind::Other, format!("Error executing command: {}. Error: {}", cmd, String::from_utf8_lossy(&output.stderr))));
    }

    Ok(())
}

pub fn get_fan_speed(temp: f64, fan_speeds: &[config::FanSpeed]) -> u8 {
    for fan_speed in fan_speeds {
        if fan_speed.temp_range[0] <= temp && temp < fan_speed.temp_range[1] {
            return fan_speed.speed;
        }
    }
    100
}
