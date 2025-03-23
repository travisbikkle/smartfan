use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub ipmi: IpmiHostInfo,
    pub fan_speeds: Vec<FanSpeed>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpmiHostInfo {
    pub host: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FanSpeed {
    pub temp_range: [f64; 2],
    pub speed: u8,
}
