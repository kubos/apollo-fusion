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

mod packets;
mod transmit;

use crate::packets::*;
use crate::transmit::*;
use failure::Error;
use kubos_app::*;
use log::*;
use nsl_simplex_s3::SimplexS3;
use std::thread;
use std::time::Duration;

struct MyApp;

impl AppHandler for MyApp {
    fn on_boot(&self, _args: Vec<String>) -> Result<(), Error> {
        // Beacon app will be started on-demand by the deployment app
        Ok(())
    }

    fn on_command(&self, _args: Vec<String>) -> Result<(), Error> {
        // TODO: Which UART for simplex?
        let simplex = SimplexS3::new("/dev/ttyS2")?;
        let telem_service = ServiceConfig::new("telemetry-service");
        let radios = Radios {
            telem_service,
            simplex,
        };

        // Spawn threads for each of the beacon messages
        debug!("Spawning OBC beacon thread");
        let obc_radios = radios.clone();
        let handle = thread::spawn(move || obc::obc_packet(obc_radios));

        debug!("Spawning temperature beacon thread");
        let temp_radios = radios.clone();
        let handle = thread::spawn(move || temperature::temp_packet(temp_radios));

        // TODO: Stay in a loop forever so the threads keep going

        if let Err(error) = handle.join() {
            error!("OBC thread panicked: {:?}", error);
        }
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let app = MyApp;
    app_main!(&app)?;

    Ok(())
}
