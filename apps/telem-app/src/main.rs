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

mod duplex;
mod eps;
mod mai400;
mod obc;
mod oem6;
mod sup_mcu;
mod telem_db;

use failure::Error;
use kubos_app::*;
use log::*;
use std::thread;
use std::time::Duration;

struct MyApp;

impl AppHandler for MyApp {
    fn on_boot(&self, _args: Vec<String>) -> Result<(), Error> {
        loop {
            if let Err(error) = duplex::get_telem() {
                error!("Error while fetching Duplex telemetry: {:?}", error);
            }

            if let Err(error) = obc::get_telem() {
                error!("Error while fetching OBC telemetry: {:?}", error);
            }

            if let Err(error) = eps::get_telem() {
                error!("Error while fetching EPS telemetry: {:?}", error);
            }

            if let Err(error) = mai400::get_telem() {
                error!("Error while fetching MAI-400 telemetry: {:?}", error);
            }

            if let Err(error) = oem6::get_telem() {
                error!("Error while fetching OEM6 telemetry: {:?}", error);
            }

            if let Err(error) = sup_mcu::get_telem() {
                error!("Error while fetching Sup MCU telemetry: {:?}", error);
            }

            thread::sleep(Duration::from_secs(60));
        }
    }

    fn on_command(&self, _args: Vec<String>) -> Result<(), Error> {
        if let Err(error) = duplex::get_telem() {
            error!("Error while fetching Duplex telemetry: {:?}", error);
        }

        if let Err(error) = obc::get_telem() {
            error!("Error while fetching OBC telemetry: {:?}", error);
        }

        if let Err(error) = eps::get_telem() {
            error!("Error while fetching EPS telemetry: {:?}", error);
        }

        if let Err(error) = mai400::get_telem() {
            error!("Error while fetching MAI-400 telemetry: {:?}", error);
        }

        if let Err(error) = oem6::get_telem() {
            error!("Error while fetching OEM6 telemetry: {:?}", error);
        }

        if let Err(error) = sup_mcu::get_telem() {
            error!("Error while fetching Sup MCU telemetry: {:?}", error);
        }

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let app = MyApp;
    app_main!(&app)?;

    Ok(())
}
