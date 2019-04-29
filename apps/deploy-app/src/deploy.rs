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
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Default deploy delay: 45 minutes
const DELAY_DEFAULT: Duration = Duration::from_secs(45 * 60);
static FW_SETENV_PATH: &'static str = "/usr/sbin/fw_setenv";

// Deployment statuses
#[derive(Debug, PartialEq)]
pub enum DeployStatus {
    RemoveBeforeFlight,
    Ready,
    AlreadyDeployed,
}

pub fn deploy() {
    // Deploy the solar panels
    let _ = try_deploy(false);

    // Start the radios
    start_radios();
}

pub fn try_deploy(force: bool) -> Result<(), Error> {
    let status = if force {
        // When deployment is requested from the ground, we want it to be completed immediately,
        // ignoring the hold time and any previous deployments
        DeployStatus::Ready
    } else {
        // Check if we've already deployed.
        // If not, wait the remaining hold time
        check_deploy()
    };

    println!("Status: {:?}", status);

    if status == DeployStatus::RemoveBeforeFlight {
        warn!("RBF active. Deployment disabled");
        bail!("RBF active. Deployment disabled");
    }

    let mut success = true;
    if status == DeployStatus::Ready {
        let mcu_service = ServiceConfig::new("pumpkin-mcu-service");
        // Deploy the panels (BIM)
        if let Err(error) = query(&mcu_service, DEPLOY_ENABLE, Some(QUERY_TIMEOUT)) {
            error!("Failed to enable deploy pin: {:?}", error);
            success = false;
        };
        thread::sleep(Duration::from_millis(100));
        if let Err(error) = query(&mcu_service, DEPLOY_ARM, Some(QUERY_TIMEOUT)) {
            error!("Failed to arm deploy pin: {:?}", error);
            success = false;
        };
        thread::sleep(Duration::from_millis(100));
        if let Err(error) = query(&mcu_service, DEPLOY_FIRE, Some(QUERY_TIMEOUT)) {
            error!("Failed to fire deploy pin: {:?}", error);
            success = false;
        };

        // Note: The `deployed` envar will be updated later after we receive verification from
        // the ground
    }

    if success {
        Ok(())
    } else {
        bail!("Deployment may have failed")
    }
}

fn start_radios() {
    let mcu_service = ServiceConfig::new("pumpkin-mcu-service");

    // Turn on radios:
    // Duplex radio
    if let Err(error) = query(&mcu_service, DUPLEX_POWER, Some(QUERY_TIMEOUT)) {
        error!("Failed to turn on duplex radio: {:?}", error);
    };
    // Simplex radio
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
    match query(&app_service, START_BEACON, Some(QUERY_TIMEOUT)) {
        Ok(msg) => {
            let success = msg
                .get("startApp")
                .and_then(|data| data.get("success").and_then(|val| val.as_bool()));

            if success == Some(true) {
                info!("Successfully started beacon app");
            } else {
                match msg.get("errors") {
                    Some(errors) => error!("Failed to start beacon app: {}", errors),
                    None => error!("Failed to start beacon app"),
                };
            }
        }
        Err(err) => {
            error!("Failed to start beacon app: {:?}", err);
        }
    }
}

fn check_deploy() -> DeployStatus {
    // See if we've already deployed
    let deployed = UBootVars::new().get_bool("deployed").unwrap_or_else(|| {
        // This variable isn't set by default, so it's okay if this called failed
        // Assume we haven't deployed yet
        false
    });

    if deployed {
        return DeployStatus::AlreadyDeployed;
    }

    // See if we're allowed to deploy
    let rbf = UBootVars::new()
        .get_bool("remove_before_flight")
        .unwrap_or_else(|| {
            error!("Failed to fetch RBF status");
            // If we can't check the status, play it safe and don't attempt deployment
            true
        });

    if rbf {
        return DeployStatus::RemoveBeforeFlight;
    }

    // Check if we need to wait before deploying
    if let Some(delay) = get_deploy_delay() {
        debug!("Starting deployment delay: {:?}", delay);
        thread::sleep(delay);
    }

    DeployStatus::Ready
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
    debug!("Current system time: {:?}", now);
    let now = match now.duration_since(UNIX_EPOCH) {
        Ok(val) => val,
        Err(err) => {
            error!("Failed to get current system time: {:?}", err);
            return None;
        }
    };

    match started {
        // We've started deployment before, so we don't need to do the full deployment delay.
        // Just pick up where we left off
        Some(value) => {
            let start_str: Vec<&str> = value.split('.').collect();
            let seconds: u64 = start_str[0].parse().unwrap_or(0);
            let nanos: u32 = start_str[1].parse().unwrap_or(0);

            if seconds == 0 && nanos == 0 {
                // If for some reason we can't parse the previous start time, skip the hold time
                // and just go straight to deployment
                error!("Failed to parse deployment start time");
                return None;
            }

            let start_time = Duration::new(seconds, nanos);

            match now.checked_sub(start_time) {
                Some(elapsed) => delay.checked_sub(elapsed),
                None => None,
            }
        }
        // If it doesn't exist, this is the first time we've started deployment.
        // save the currSysTime into the new uboot envar
        None => {
            if set_boot_var(
                "deploy_start",
                &format!("{}.{}", now.as_secs(), now.subsec_nanos()),
            )
            .is_ok()
            {
                // Use the full deployment delay
                Some(delay)
            } else {
                // If we failed to set the start time, we should assume that we've failed to set it
                // before and this isn't actually the first time we've been here.
                // In this case, we should skip the hold time and go straight to deployment
                None
            }
        }
    }
}

pub fn set_boot_var(name: &str, value: &str) -> Result<(), Error> {
    let result = Command::new(FW_SETENV_PATH).args(&[name, value]).output()?;

    if !result.status.success() {
        error!(
            "Failed to set envar '{}': RC={:?}, stderr='{}'",
            name,
            result.status.code(),
            ::std::str::from_utf8(&result.stderr).unwrap_or("")
        );
        bail!("Failed to set envar");
    }

    Ok(())
}
