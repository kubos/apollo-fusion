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

// OBC Housekeeping Application
//
// Every hour:
// - Clean out any telemetry database entries which are older than a week
// - Check for excessive disk or RAM usage
// - Check for corruption in the user data partition
// - Check for OBC resets
// - Ping all the services

mod check_fs;
mod check_mem;
mod check_reset;
mod clean_db;
mod ping;

use failure::{bail, err_msg, Error};
use kubos_app::*;
use log::*;
use std::cell::Cell;
use std::thread;
use std::time::Duration;

pub const QUERY_TIMEOUT: Duration = Duration::from_millis(200);
// TODO: Figure out correct downlink port number
pub const DOWNLINK_PORT: u16 = 14011;

struct MyApp {
    // Used to help detect system reboots. Gets set the first time the `check_reset` function is
    // called
    active_flag: Cell<bool>,
}

impl AppHandler for MyApp {
    fn on_boot(&self, _args: Vec<String>) -> Result<(), Error> {
        loop {
            // Delete everything from the database that's more than a week old (60*60*24*7)
            if let Err(error) = clean_db::clean_db(604_800.0) {
                error!("Error while cleaning telemetry database: {:?}", error);
            }

            // Check RAM and disk usage
            if let Err(error) = check_mem::check_mem() {
                error!("Error while checking memory: {:?}", error);
            }

            // Check for file system corruption
            if let Err(error) = check_fs::check_fs() {
                error!("Error while checking filesystem {:?}", error);
            }

            // Check for system reset
            if let Err(error) = check_reset::check_reset(&self.active_flag) {
                error!("Error while checking system reset: {:?}", error);
            }

            // Ping all services
            match ping::ping_services() {
                Ok(0) => info!("Successfully pinged all services"),
                Ok(count) => warn!("Failed to ping {} services", count),
                Err(error) => error!("Error while pinging the services: {:?}", error),
            }

            thread::sleep(Duration::from_secs(60));
        }
    }

    fn on_command(&self, _args: Vec<String>) -> Result<(), Error> {
        println!("OnCommand logic");
        // Nice to have (todo): Add ability to manually remove more of the telemetry database entries
        // (ex. only retain the last day's worth of data)
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let app = MyApp {
        active_flag: Cell::new(false),
    };
    app_main!(&app, log::LevelFilter::Info)?;

    Ok(())
}
