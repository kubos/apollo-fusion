// Copyright (C) 2018 Kubos Corporation. All rights reserved.
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use kubos_service::Config;
use kubos_system;
use serde_json;

pub const GPS_SERVICE: &'static str = "novatel-oem6-service";
pub const EPS_SERVICE: &'static str = "clyde-3g-eps-service";
pub const MCU_SERVICE: &'static str = "pumpkin-mcu-service";

lazy_static! {
    static ref SERVICES: HashMap<String, String> = {
        let mut map: HashMap<String, String> = HashMap::new();
        for service in &[GPS_SERVICE, EPS_SERVICE, MCU_SERVICE] {
            map.insert(
                String::from(*service),
                String::from(Config::new(*service).hosturl()),
            );
        }
        map
    };
}

pub fn get(name: &str) -> Option<&str> {
    match SERVICES.get(&String::from(name)) {
        Some(val) => Some(&val),
        None => None,
    }
}

pub fn query(name: &str, query: &str, duration_secs: u64) -> Result<serde_json::Value, String> {
    match get(name) {
        Some(hosturl) => {
            match kubos_system::query(hosturl, query, Some(Duration::from_secs(duration_secs))) {
                Ok(value) => Ok(value.clone()),
                Err(e) => Err(format!("{}", e)),
            }
        }
        None => Err(format!("Couldn't find service named {}", name)),
    }
}
