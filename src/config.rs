use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub notify_urls: Vec<String>,
    pub context_length: usize,
    pub context_window_duration: String,
    pub rate_limit_msg_per_minute: usize,
    pub cool_down_duration: String,
}

pub struct ParsedDurations {
    pub context_window: Duration,
    pub cool_down: Duration,
}

pub fn load_config(path: &Path) -> Result<(AppConfig, ParsedDurations)> {
    if !path.exists() {
        return Err(anyhow!("Configuration file not found: {:?}", path));
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    let config: AppConfig = serde_yaml::from_str(&text)
        .with_context(|| format!("failed to parse YAML config {}", path.display()))?;
    
    let durations = ParsedDurations {
        context_window: parse_duration_str(&config.context_window_duration)?,
        cool_down: parse_duration_str(&config.cool_down_duration)?,
    };

    Ok((config, durations))
}

fn parse_duration_str(s: &str) -> Result<Duration> {
    let re = Regex::new(r"^(\d+)(m|s)$")?;
    let caps = re.captures(s).ok_or_else(|| anyhow!("Invalid duration format: {}", s))?;
    let val: u64 = caps[1].parse()?;
    match &caps[2] {
        "m" => Ok(Duration::from_secs(val * 60)),
        "s" => Ok(Duration::from_secs(val)),
        _ => Err(anyhow!("Unsupported time unit")),
    }
}
