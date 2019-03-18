//
// Copyright (C) 2019 Kubos Corporation
//
// Licensed under the Apache License, Version 2.0 (the "License")
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use crate::graphql::*;
use failure::{bail, Error};
use kubos_app::*;
use kubos_system::*;
use log::*;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Default deploy delay: 45 minutes
const DELAY_DEFAULT: Duration = Duration::from_secs(2700);
static FW_SETENV_PATH: &'static str = "/usr/sbin/fw_setenv";

pub fn deploy() -> Result<(), Error> {
    // Check if we've already deployed.
    // If not, wait the remaining holdtime
    let status = check_deploy()?;
    
    if status == DeployStatus::Ready {
        // Deploy the panels (BIM)
        set_deployed();
    }
    
    // Turn on radios
    // - TODO: NSL Duplex
    // - NSL Simplex
    let mcu_service = ServiceConfig::new("pumpkin-mcu-service");
    if let Err(error) = query(&mcu_service, SIMPLEX_POWER, Some(QUERY_TIMEOUT)) {
        error!("Failed to turn on simplex radio: {:?}", error);
    };
    if let Err(error) = query(&mcu_service, SIMPLEX_COMM, Some(QUERY_TIMEOUT)) {
        error!("Failed to set simplex UART: {:?}", error);
    }
    if let Err(error) = query(&mcu_service, SIMPLEX_PASS, Some(QUERY_TIMEOUT)) {
        error!("Failed to enable simplex communication: {:?}", error);            
    };
    
    // Start transmitting H&S beacon
    let app_service = ServiceConfig::new("app-service");
    if let Err(error) = query(&app_service, START_BEACON, Some(QUERY_TIMEOUT)) {
        error!("Failed to start beacon app: {:?}", error);
    };
        
    Ok(())
}

fn check_deploy() -> Result<DeployStatus, Error> {
    // See if we've already deployed
    let deployed = UBootVars::new().get_bool("deployed").unwrap_or_else(|| {
        error!("Failed to fetch deployment status");
        false
    });

    if deployed == true {
        return Ok(DeployStatus::AlreadyDeployed);
    }

    // Check if we need to wait before deploying
    if let Some(delay) = get_deploy_delay() {
        info!("Starting deployment delay: {:?}", delay);
        sleep(delay);
    }

    Ok(DeployStatus::Ready)
}

fn set_deployed() {
    let result = set_boot_var("deployed", "true");
    if !result.is_ok() {
        error!("Failed to set deployment status");
    }
}

fn set_boot_var(name: &str, value: &str) -> Result<String, Error> {
    let output = Command::new(FW_SETENV_PATH)
        .args(&[name, value])
        .output()
        .expect(&format!("Failed to execute {}", FW_SETENV_PATH));

    if !output.status.success() {
        error!("Failed to set envar: {}", name);
        bail!("Failed to set var")
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

fn get_deploy_delay() -> Option<Duration> {
    // Get uboot envar with time deployment started
    let started = UBootVars::new().get_str("deploy_start");
    
    // Get the configuration options for the service out of the `config.toml` file
    let config = Config::new("deployment");
    
    // Get the desired delay amount
    let delay = config
        .get("deploy-delay")
        .and_then(|val| val.as_integer())
        .and_then(|val| Some(Duration::from_secs(val as u64)))
        .unwrap_or(DELAY_DEFAULT);
        
    // Get current system time
    let now = SystemTime::now();
    info!("Current system time: {:?}", now);
    let now = now.duration_since(UNIX_EPOCH).unwrap();

    match started {
        // We've started deployment before, so we don't need to do the full deployment delay.
        // Just pick up where we left off
        Some(value) => {
            let start_str: Vec<&str> = value.split('.').collect();
            let seconds: u64 = start_str[0].parse().unwrap_or(0);
            let nanos: u32 = start_str[1].parse().unwrap_or(0);

            let start_time = if seconds == 0 && nanos == 0 {
                error!("Failed to parse deployment start time");
                now.clone()
            } else {
                Duration::new(seconds, nanos)
            };

            match now.checked_sub(start_time) {
                Some(elapsed) => delay.checked_sub(elapsed),
                None => None,
            }
        }
        // If it doesn't exist, this is the first time we've started deployment.
        // save the currSysTime into the new uboot envar
        None => {
            let result = set_boot_var(
                "deploy_start",
                &format!("{}.{}", now.as_secs(), now.subsec_nanos()),
            );
            if !result.is_ok() {
                error!("Failed to set deployment start time");
            }
            Some(delay)
        }
    }
}

// Deployment statuses
#[derive(PartialEq)]
pub enum DeployStatus {
    Ready,
    AlreadyDeployed,
}