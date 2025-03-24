use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

use tokio::sync::mpsc::{Receiver, Sender};
use derive_more::Display;
use log::Level;

pub mod config;
pub mod constants;
pub mod fan;
pub mod tui;

pub use constants::*;

#[derive(Debug, Display)]
pub enum Message {
    #[display("{}: {}", _0, _1)]
    Log(Level, String), // log
    #[display("Command: {}", _0)]
    Command(String), // error
    #[display("Ipmi: temp: {} speed {}", _0, _1)]
    Ipmi(f64, u8),   // temperature, speed
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
        send_to_ui.send(Message::Log(Level::Error, format!("{} not exists.", config_path))).await.expect("send message to ui successfully");
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
        match fan::get_temperature_and_cpu_num(&ipmi_tool_cmd) {
            Ok(Some((temp, cpu_num))) => {
                let speed = fan::get_fan_speed(temp, &config.fan_speeds);
                if fan::set_fan_speed(speed, &ipmi_tool_cmd, cpu_num, &mut cpu2_fan_speed_set) {
                    // show log on tui
                    //log::info!("Set fan speed to {}% for CPU temperature {}°C", speed, temp);
                    send_to_ui.send(Message::Ipmi(temp, speed)).await.expect("send message to ui successfully");
                }
            }
            Err(e) => send_to_ui.send(Message::Log(Level::Error, e.to_string())).await.expect("send message to ui successfully"),
            _ => { println!("fan: failed to get temperature and cpu number"); }
        }
        // tokio async
        tokio::time::sleep(Duration::from_secs(10)).await;
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
