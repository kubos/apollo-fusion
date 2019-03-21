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
mod graphql;

use crate::deploy::*;
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

        // TODO: Maybe just move GPS/ADCS initialization into their housekeeping apps
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
        match query(&oem_service, OEM_SET_LOGS, Some(QUERY_TIMEOUT)) {
            Ok(msg) => {
                let success = msg
                    .get("configureHardware")
                    .and_then(|data| data.get("success").and_then(|val| val.as_bool()));

                if success == Some(true) {
                    info!("Successfully configured OEM logging");
                } else {
                    match msg.get("errors") {
                        Some(errors) => error!("Failed to configure OEM logging: {}", errors),
                        None => error!("Failed to configure OEM logging"),
                    };
                }
            }
            Err(err) => {
                error!("Failed to configure OEM logging: {}", err);
            }
        }

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

        // Kick off ADCS housekeeping app
        let app_service = ServiceConfig::new("app-service");
        match query(&app_service, START_ADCS, Some(QUERY_TIMEOUT)) {
            Ok(msg) => {
                let success = msg
                    .get("startApp")
                    .and_then(|data| data.get("success").and_then(|val| val.as_bool()));

                if success == Some(true) {
                    info!("Successfully started ADCS housekeeping app");
                } else {
                    match msg.get("errors") {
                        Some(errors) => error!("Failed to start ADCS housekeeping app: {}", errors),
                        None => error!("Failed to start ADCS housekeeping app"),
                    };
                }
            }
            Err(err) => {
                error!("Failed to start ADCS housekeeping app: {:?}", err);
            }
        }

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
