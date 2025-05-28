use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Time intervals for K-line data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInterval {
    #[serde(rename = "1s")]
    Second1,
    #[serde(rename = "1m")]
    Minute1,
    #[serde(rename = "5m")]
    Minute5,
    #[serde(rename = "15m")]
    Minute15,
    #[serde(rename = "1h")]
    Hour1,
}

impl FromStr for TimeInterval {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1s" => Ok(Self::Second1),
            "1m" => Ok(Self::Minute1),
            "5m" => Ok(Self::Minute5),
            "15m" => Ok(Self::Minute15),
            "1h" => Ok(Self::Hour1),
            _ => Err(format!("Invalid time interval: {}", s)),
        }
    }
}

impl TimeInterval {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Second1 => "1s",
            Self::Minute1 => "1m",
            Self::Minute5 => "5m",
            Self::Minute15 => "15m",
            Self::Hour1 => "1h",
        }
    }

    /// Get duration in seconds
    pub fn duration_seconds(&self) -> u64 {
        match self {
            Self::Second1 => 1,
            Self::Minute1 => 60,
            Self::Minute5 => 300,
            Self::Minute15 => 900,
            Self::Hour1 => 3600,
        }
    }
}
