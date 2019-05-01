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

/*
- Telemetry database: Clean up old entries
- Check for disk/ram issues
- Check for filesystem corruption (everything still read/writeable?)
- Check for OBC reset - generate error log if there was a reset
- Ping services
*/

use failure::{bail, err_msg, Error};
use kubos_app::*;
use log::*;
use std::cell::Cell;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub const QUERY_TIMEOUT: Duration = Duration::from_millis(100);

struct MyApp {
    active_flag: Cell<bool>   
}

impl AppHandler for MyApp {
    fn on_boot(&self, _args: Vec<String>) -> Result<(), Error> {
        loop {
            // Delete everything from the database that's more than a week old (60*60*24*7)
            if let Err(error) = clean_db(604800.0) {
                error!("Error while cleaning telemetry database: {:?}", error);
            }

            if let Err(error) = check_mem() {
                error!("Error while checking memory: {:?}", error);
            }
            
            if let Err(error) = check_fs() {
                error!("Error while checking filesystem {:?}", error);
            }

            if let Err(error) = check_reset(&self.active_flag) {
                error!("Error while checking system reset: {:?}", error);
            }

            match ping_services() {
                Ok(0) => info!("Successfully pinged all services"),
                Ok(count) => warn!("Failed to ping {} services", count),
                Err(error) => error!("Error while pinging the services: {:?}", error),
            }

            thread::sleep(Duration::from_secs(6));
        }
    }

    fn on_command(&self, _args: Vec<String>) -> Result<(), Error> {
        println!("OnCommand logic");
        // TODO: Add ability to manually remove more of the telemetry database entries
        // (ex. only retain the last day's worth of data)
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let app = MyApp { active_flag: Cell::new(false) };
    app_main!(&app, log::LevelFilter::Info)?;

    Ok(())
}

fn clean_db(age: f64) -> Result<(), Error> {
    // Get the current timestamp
    let time = time::now_utc().to_timespec();
    // Convert it to fractional seconds and calculate the timestamp for the requested age
    let timestamp = time.sec as f64 + (f64::from(time.nsec) / 1_000_000_000.0) - age;
    
    // Request that the telemetry service remove everything older than a week
    let telem_service = ServiceConfig::new("telemetry-service");
    
    let request = format!(r#"mutation {{
        delete(timestampLe: {}) {{
            success,
            errors,
            entriesDeleted
        }}
    }}"#, timestamp);
    
    let response = query(&telem_service, &request, Some(Duration::from_millis(200)))?;
    
    // Check the results
    let data = response.get("delete").ok_or(err_msg("Failed to get delete response"))?;
    let success = data.get("success").and_then(|val| val.as_bool());
                
    if success == Some(true) {
        let count = data.get("entriesDeleted").and_then(|val| val.as_u64()).unwrap_or(0);
        info!("Deleted {} telemetry entries", count);
    } else {
        match data.get("errors") {
            Some(errors) => bail!("Failed to delete telemetry entries: {}", errors),
            None => bail!("Failed to delete telemetry entries"),
        };
    }
    
    Ok(())
}

const OBC_TELEMETRY: &str = r#"{
    memInfo {
        available
    }
}"#;

// Taken from /proc/meminfo on a BBB
const MEM_TOTAL: u64 = 515_340;

fn check_mem() -> Result<(), Error> {
    // Check RAM usage
    let service = ServiceConfig::new("monitor-service");

    let result = query(&service, OBC_TELEMETRY, Some(Duration::from_secs(1)))?;

    let mem = result["memInfo"]["available"].as_u64().unwrap_or(0);
    
    // Convert to percentage in use, since that's an easier number to work with
    let ram_in_use = 100 - mem * 100 / MEM_TOTAL;
    
    // TODO: Decide on thresholds
    if ram_in_use < 50 {
        info!("RAM usage nominal: {}%", ram_in_use);
    } else if ram_in_use < 70 {
        info!("RAM usage high, but acceptable: {}%", ram_in_use);
    } else if ram_in_use < 80 {
        warn!("RAM usage high: {}%", ram_in_use);
    } else {
        error!("RAM usage too high: {}%. Triggering reboot", ram_in_use);
        // TODO: reboot
    }
    
    // Check disk space usage
    // Get the % of the user data partition that's free
    //
    // Since we're using the MBM2, we'll need to check both of the possible disk names.
    // If the SD card is present, then the eMMC will be mmc1. Otherwise, it will be mmc0.
    //
    // Note: I tried to just use a wildcard ("/dev/mmcblk*p4"), but couldn't get the correct
    // output for some reason, so we're doing this the long way.
    let disk_in_use = if let Ok(output1) = Command::new("df").arg("/dev/mmcblk1p4").output() {
        let stdout = if output1.stderr.is_empty() {
            output1.stdout
        } else if let Ok(output0) = Command::new("df").arg("/dev/mmcblk0p4").output() {
            if output0.stderr.is_empty() {
                output0.stdout
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let mut slices = stdout.rsplit(|&elem| elem == b' ');

        // The last entry is the mount point (/home)
        slices.next();
        // The second to last entry is the percent in use
        let temp = slices.next();
        // Convert it to a useable number
        let percent = temp
            .unwrap_or(&[])
            .iter()
            .filter_map(|&elem| {
                if elem.is_ascii_digit() {
                    Some(elem as char)
                } else {
                    None
                }
            })
            .collect::<String>();

        percent.parse::<u8>().unwrap_or(100)
    } else {
        error!("Failed to get current disk usage info");
        100
    };
    
    if disk_in_use < 50 {
        info!("Disk usage nominal: {}%", disk_in_use);
    } else if disk_in_use < 70 {
        info!("Disk usage high, but acceptable: {}%", disk_in_use);
    } else if disk_in_use < 80 {
        warn!("Disk usage high: {}%", disk_in_use);
    } else {
        error!("Disk usage too high: {}%. Triggering cleanup", disk_in_use);
        // Delete everything from the database that's more than a day old (60*60*24)
        clean_db(86400.0)?;
    }
    
    Ok(())
}

fn check_fs() -> Result<(), Error> {
    // TODO: Check that the user data partition is still writeable.
    Ok(())
}

fn check_reset(active_flag: &Cell<bool>) -> Result<(), Error> {
    if !active_flag.get() {
        // If we're here, that means one of two things:
        // 1. The system just started up
        // 2. This app was restarted
        
        // Get the current uptime
        let uptime = if let Ok(output) = Command::new("cat").arg("/proc/uptime").output() {
            if !output.stderr.is_empty() {
                bail!("Failed to get system uptime: {}", ::std::str::from_utf8(&output.stderr).unwrap_or("n/a"));
            }
    
            let mut slices = output.stdout.split(|&elem| elem == b' ');
    
            // The first entry is the overall system uptime
            let temp = slices.next().ok_or(err_msg("Failed to get system uptime"))?;
            // Convert it to a useable number
            let uptime = ::std::str::from_utf8(&temp)?;
            uptime.parse::<f32>()?
        } else {
            bail!("Failed to get system uptime");  
        };
        
        println!("Uptime: {}", uptime);
        
        // If the uptime is less than 30 seconds, we'll assume that the entire system was restarted,
        // rather than just this app
        if uptime < 30.0 {
            error!("System reset observed");
        }
        
        // Mark the flag as true for the next time we're here
        active_flag.set(true);
    }
    
    Ok(())
}

fn ping_services() -> Result<u8, Error> {
    // Ping all the services to make sure they're still running
    // TODO: What do we do if they're not up? Nothing? In theory Monit should have a handle on things

    let mut bad_count = 0;

    // Core services:
    // TODO: comms service
    // TODO: file transfer service?
    // TODO: shell service?
    if ping("app-service").is_err() {
        bad_count += 1;
    }
    if ping("monitor-service").is_err() {
        bad_count += 1;
    }
    if ping("telemetry-service").is_err() {
        bad_count += 1;
    }

    // Hardware services:
    if ping("mai400-service").is_err() {
        bad_count += 1;
    }
    if ping("clyde-3g-eps-service").is_err() {
        bad_count += 1;
    }
    if ping("novatel-oem6-service").is_err() {
        bad_count += 1;
    }
    if ping("pumpkin-mcu-service").is_err() {
        bad_count += 1;
    }

    Ok(bad_count)
}

fn ping(service: &str) -> Result<(), Error> {
    let config = ServiceConfig::new(service);
    match query(&config, "{ping}", Some(QUERY_TIMEOUT)) {
        Ok(data) => match data["ping"].as_str() {
            Some("pong") => Ok(()),
            other => {
                error!("Got bad result from {}: {:?}", service, other);
                bail!("Bad result");
            }
        },
        Err(err) => {
            error!("Failed to ping {}: {:?}", service, err);
            Err(err)
        }
    }
}
