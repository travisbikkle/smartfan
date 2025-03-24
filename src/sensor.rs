use std::str::FromStr;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct SensorResult {
    pub sensor_name: String,
    pub value: Option<f64>,       // 当前值（"na" 转换为 None）
    pub unit: Option<String>,     // 单位（可能为空）
    pub status: Option<String>,   // 状态（"na" 表示不可用）
    pub thresholds: Thresholds,  // 封装所有阈值
}

#[derive(Debug, PartialEq)]
pub struct Thresholds {
    pub lnr: Option<f64>,  // Lower Non-Recoverable
    pub lc: Option<f64>,   // Lower Critical
    pub lnc: Option<f64>,  // Lower Non-Critical
    pub unc: Option<f64>,  // Upper Non-Critical
    pub uc: Option<f64>,   // Upper Critical
    pub unr: Option<f64>,  // Upper Non-Recoverable
}

impl SensorResult {
    /// 构建方法，处理原始字符串行
    pub fn from_line(line: &str) -> Result<Self, ParseError> {
        let columns: Vec<&str> = line.split('|').map(|s| s.trim()).collect();

        if columns.len() < 10 {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            sensor_name: columns[0].to_string(),
            value: parse_optional_f64(columns[1]),
            unit: parse_optional_string(columns[2]),
            status: parse_optional_string(columns[3]),
            thresholds: Thresholds {
                lnr: parse_optional_f64(columns[4]),
                lc: parse_optional_f64(columns[5]),
                lnc: parse_optional_f64(columns[6]),
                unc: parse_optional_f64(columns[7]),
                uc: parse_optional_f64(columns[8]),
                unr: parse_optional_f64(columns[9]),
            },
        })
    }
}

// 辅助函数：处理可选数值字段
fn parse_optional_f64(s: &str) -> Option<f64> {
    s.parse().ok()
}

// 辅助函数：处理可选字符串字段（过滤 "na"）
fn parse_optional_string(s: &str) -> Option<String> {
    match s {
        "na" | "" => None,
        _ => Some(s.to_string())
    }
}

// 错误处理（参考网页3的错误处理模式）
#[derive(Debug)]
pub enum ParseError {
    InvalidFormat,
    ParseFailure(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "Invalid sensor data format"),
            ParseError::ParseFailure(s) => write!(f, "Parse failed: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sensor() {
        let line = "CPU1_Temp | 85.0 | degrees C | ok | na | na | na | 90.0 | 95.0 | 100.0";
        let sensor = SensorResult::from_line(line).unwrap();

        assert_eq!(sensor.sensor_name, "CPU1_Temp");
        assert_eq!(sensor.value, Some(85.0));
        assert_eq!(sensor.unit, Some("degrees C".to_string()));
        assert_eq!(sensor.status, Some("ok".to_string()));
        assert_eq!(sensor.thresholds.uc, Some(95.0));
    }
}