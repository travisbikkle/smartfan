use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;
use chrono::Local;
use tokio::sync::mpsc::{Receiver, Sender};
use derive_more::Display;
use log::Level;

pub mod config;
pub mod constants;
pub mod fan;
pub mod tui;
pub mod sensor;

pub use constants::*;

#[derive(Debug, Display)]
pub enum Message {
    #[display("{}: {}", _0, _1)]
    Log(String, Level, String), // log
    #[display("Command: {}", _0)]
    Command(String), // error
    #[display("Ipmi: temp: {} speed {}", _0, _1)]
    SetFanSpeed(String, f64, u8),   // temperature, speed
    #[display("Ipmi: cpu: {} speed {}", _1.0, _2.len())]
    GotCpuAndFansSpeed(String, (usize, usize), Vec<(String, f64)>),   // temperature, speed
}

impl Message {
    pub fn build_log(level: Level, msg: String) -> Message {
        let now = Local::now();
        let time_str = now.format("%H:%M:%S").to_string();
        Message::Log(time_str, level, msg)
    }
}

#[derive(Debug, Display)]
pub enum UIMessage {
    RestartLoop,
}

pub fn load_config(config_path: &str) -> config::Config {
    let mut file = File::open(config_path).expect("Failed to open config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read config file");

    let config: config::Config =
        serde_yaml::from_str(&contents).expect("Failed to parse config file");
    config
}

pub async fn init_loop(send_to_ui: Sender<Message>, receive_from_ui: Receiver<UIMessage>) {
    let in_band = false;
    // if in_band {
    //     log::info!("Running with in-band mode");
    // } else {
    //     log::info!("Running with out-band mode");
    // }

    let config_path = format!("{}/HR650X.yaml", std::env::current_dir().unwrap().display());
    if !std::fs::metadata(config_path.clone()).is_ok() {
        send_to_ui.send(Message::build_log(Level::Error, format!("{} not exists.", config_path))).await.expect("send message to ui successfully");
        return;
    }
    let config: config::Config = load_config(&config_path);

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

        match fan::get_all_sensor_data(&ipmi_tool_cmd) {
            Ok(sensor_data) => {
                let now = Local::now();
                let time_str = now.format("%H:%M:%S").to_string();
                let (active_cpu_nums, max) = fan::get_active_cpu_num(&sensor_data);
                let max_temperature = fan::get_max_temperature(&sensor_data);
                let speed = fan::get_fan_speed(max_temperature, &config.fan_speeds);
                let all_fans_speed = fan::get_fans_speed(&sensor_data);
                send_to_ui.send(Message::build_log(Level::Info, format!("GotCpuAndFansSpeed, active cpu num: {}, max sockets num: {}, fans sensor num: {}", active_cpu_nums, max, all_fans_speed.len()))).await.expect("send message to ui successfully");
                send_to_ui.send(Message::GotCpuAndFansSpeed(time_str.clone(), (active_cpu_nums, max), all_fans_speed)).await.expect("send message to ui successfully");
                match fan::set_fan_speed(speed, &ipmi_tool_cmd, active_cpu_nums, &mut cpu2_fan_speed_set) {
                    Ok(()) => {
                        send_to_ui.send(Message::build_log(Level::Info, format!("SetFanSpeed, temp: {}, speed: {}", max_temperature, speed))).await.expect("send message to ui successfully");
                        send_to_ui.send(Message::SetFanSpeed(time_str, max_temperature, speed)).await.expect("send message to ui successfully");
                    }
                    Err(e) => {
                        send_to_ui.send(Message::build_log(Level::Error, e.to_string())).await.expect("send message to ui successfully");
                    }
                }
            }
            Err(e) => send_to_ui.send(Message::build_log(Level::Error, e.to_string())).await.expect("send message to ui successfully"),
        }

        // tokio async
        tokio::time::sleep(Duration::from_millis(15000)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config_path = format!(
            "{}/{}/HR650X.yaml",
            std::env::current_dir().unwrap().display(),
            "scripts"
        );
        let config = load_config(&config_path);
        assert_eq!(config.ipmi.username, "changeme");
    }
}
