use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChartType {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    FourHours,
    OneDay,
}

impl FromStr for ChartType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1m" => Ok(ChartType::OneMinute),
            "5m" => Ok(ChartType::FiveMinutes),
            "15m" => Ok(ChartType::FifteenMinutes),
            "30m" => Ok(ChartType::ThirtyMinutes),
            "1h" => Ok(ChartType::OneHour),
            "4h" => Ok(ChartType::FourHours),
            "1d" => Ok(ChartType::OneDay),
            _ => Err(format!("Invalid chart type: {}", s)),
        }
    }
}

impl fmt::Display for ChartType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChartType::OneMinute => write!(f, "1m"),
            ChartType::FiveMinutes => write!(f, "5m"),
            ChartType::FifteenMinutes => write!(f, "15m"),
            ChartType::ThirtyMinutes => write!(f, "30m"),
            ChartType::OneHour => write!(f, "1h"),
            ChartType::FourHours => write!(f, "4h"),
            ChartType::OneDay => write!(f, "1d"),
        }
    }
}
