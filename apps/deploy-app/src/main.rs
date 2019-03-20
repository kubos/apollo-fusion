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

mod deploy;
mod detumble;
mod graphql;

use crate::deploy::*;
use crate::detumble::*;
use crate::graphql::*;

use failure::Error;
use kubos_app::*;
use log::*;
use std::thread;

struct MyApp;

impl AppHandler for MyApp {
    // Q: What do we want to do if any of this fails? We could try a limited number of reboots.
    // Or just log the error and keep going.
    fn on_boot(&self, _args: Vec<String>) -> Result<(), Error> {
        // Kick off hold time countdown, then go through deployment logic
        let deploy_handle = thread::spawn(deploy);

        // Turn on GPS
        let mcu_service = ServiceConfig::new("pumpkin-mcu-service");
        if let Err(error) = query(&mcu_service, OEM_POWER, Some(QUERY_TIMEOUT)) {
            error!("Failed to turn on OEM: {:?}", error);
        };
        if let Err(error) = query(&mcu_service, OEM_COMM, Some(QUERY_TIMEOUT)) {
            error!("Failed to set OEM UART: {:?}", error);
        }
        if let Err(error) = query(&mcu_service, OEM_PASS, Some(QUERY_TIMEOUT)) {
            error!("Failed to enable OEM communication: {:?}", error);
        };
        // Set up OEM log messages that we do/don't want
        // (Position data + error messages)
        let oem_service = ServiceConfig::new("novatel-oem6-service");
        if let Err(error) = query(&oem_service, OEM_SET_LOGS, Some(QUERY_TIMEOUT)) {
            error!("Failed to enable OEM communication: {:?}", error);
        };

        // Turn on ADCS. It will automatically go into detumble mode
        if let Err(error) = query(&mcu_service, MAI_POWER, Some(QUERY_TIMEOUT)) {
            error!("Failed to turn on MAI-400: {:?}", error);
        };
        if let Err(error) = query(&mcu_service, MAI_COMM, Some(QUERY_TIMEOUT)) {
            error!("Failed to set MAI-400 UART: {:?}", error);
        }
        if let Err(error) = query(&mcu_service, MAI_PASS, Some(QUERY_TIMEOUT)) {
            error!("Failed to enable MAI-400 communication: {:?}", error);
        };

        // Wait for the sat to finish detumbling and for the ADCS to be put into normal mode
        // (may take up to 12 hours)
        detumble_wait();

        // Wait for deployment to finish before exiting
        if let Err(error) = deploy_handle.join() {
            error!("Deploy thread panicked: {:?}", error);
        }

        Ok(())
    }

    fn on_command(&self, _args: Vec<String>) -> Result<(), Error> {
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let app = MyApp;
    app_main!(&app)?;

    Ok(())
}
