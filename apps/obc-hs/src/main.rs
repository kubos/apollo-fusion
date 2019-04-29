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

use failure::{bail, Error};
use kubos_app::*;
use log::*;
use serde_json::Value;
use std::thread;
use std::time::Duration;

pub const QUERY_TIMEOUT: Duration = Duration::from_millis(100);

struct MyApp;

impl AppHandler for MyApp {
    fn on_boot(&self, _args: Vec<String>) -> Result<(), Error> {
        loop {
            /*
            if let Err(error) = clean_db() {
                error!("Error while cleaning telemetry database: {:?}", error);
            }

            if let Err(error) = check_mem() {
                error!("Error while checking memory: {:?}", error);
            }

            if let Err(error) = check_fs() {
                error!("Error while checking filesystem {:?}", error);
            }

            if let Err(error) = check_reset() {
                error!("Error while checking system reset: {:?}", error);
            }
            */

            match ping_services() {
                Ok(0) => info!("Successfully pinged all services"),
                Ok(count) => warn!("Failed to ping {} services", count),
                Err(error) => error!("Error while pinging the services: {:?}", error),
            }

            thread::sleep(Duration::from_secs(60));
        }
    }

    fn on_command(&self, _args: Vec<String>) -> Result<(), Error> {
        println!("OnCommand logic");
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let app = MyApp;
    app_main!(&app, log::LevelFilter::Info)?;

    Ok(())
}

fn clean_db() -> Result<(), Error> {
    unimplemented!();
}

fn check_mem() -> Result<(), Error> {
    unimplemented!();
}

fn check_fs() -> Result<(), Error> {
    unimplemented!();
}

fn check_reset() -> Result<(), Error> {
    unimplemented!();
}

fn ping_services() -> Result<u8, Error> {
    // Ping all the services to make sure they're still running
    // TODO: What do we do if they're not up? Nothing? In theory Monit should have a handle on things

    let mut bad_count = 0;

    // Core services:
    // TODO: comms service
    // TODO: file transfer service?
    // TODO: shell service?
    if ping_service("app-service").is_err() {
        bad_count += 1;
    }
    if ping_service("monitor-service").is_err() {
        bad_count += 1;
    }
    if ping_service("telemetry-service").is_err() {
        bad_count += 1;
    }

    // Hardware services:
    if ping_service("mai400-service").is_err() {
        bad_count += 1;
    }
    if ping_service("clyde-3g-eps-service").is_err() {
        bad_count += 1;
    }
    if ping_service("novatel-oem6-service").is_err() {
        bad_count += 1;
    }
    if ping_service("pumpkin-mcu-service").is_err() {
        bad_count += 1;
    }

    Ok(bad_count)
}

fn ping_service(service: &str) -> Result<(), Error> {
    let config = ServiceConfig::new(service);
    match query(&config, "{ping}", Some(QUERY_TIMEOUT)) {
        Ok(data) => match data["ping"].as_str() {
            Some("pong") => Ok(()),
            other => {
                error!("Got bad result from {}: {:?}", service, other);
                bail!("Bad result");
            }
        },
        Ok(other) => {
            error!("Got bad result from {}: {:?}", service, other);
            bail!("Bad result");
        }
        Err(err) => {
            error!("Failed to ping {}: {:?}", service, err);
            Err(err)
        }
    }
}
