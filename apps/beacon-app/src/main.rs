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
        let mut handles = vec![];
        // TODO: Which UART for simplex?
        let simplex = SimplexS3::new("/dev/ttyS2")?;
        let telem_service = ServiceConfig::new("telemetry-service");
        let radios = Radios {
            telem_service,
            simplex,
        };

        // Spawn threads for each of the beacon messages
        /*
        let obc_radios = radios.clone();
        let handle = thread::spawn(move || obc::obc_packet(obc_radios));
        info!("Spawning OBC beacon thread: {:?}", handle.thread().id());
        handles.push(handle);

        let temp_radios = radios.clone();
        let handle = thread::spawn(move || temperature::temp_packet(temp_radios));
        info!(
            "Spawning temperature beacon thread: {:?}",
            handle.thread().id()
        );
        handles.push(handle);

        let supmcu_radios = radios.clone();
        let handle = thread::spawn(move || supmcu::supmcu_packet(supmcu_radios));
        info!("Spawning supMCU beacon thread: {:?}", handle.thread().id());
        handles.push(handle);

        let gps_radios = radios.clone();
        let handle = thread::spawn(move || gps::gps_packet(gps_radios));
        info!("Spawning GPS beacon thread: {:?}", handle.thread().id());
        handles.push(handle);

        let adcs_radios = radios.clone();
        let handle = thread::spawn(move || adcs::adcs_packet(adcs_radios));
        info!("Spawning ADCS beacon thread: {:?}", handle.thread().id());
        handles.push(handle);
        */

        let power_radios = radios.clone();
        let handle = thread::spawn(move || power::power_packet(power_radios));
        info!("Spawning power beacon thread: {:?}", handle.thread().id());
        handles.push(handle);

        // Wait indefinitely for all the threads to exit (which they shouldn't do unless something
        // goes wrong)
        for handle in handles {
            let id = handle.thread().id();
            if let Err(error) = handle.join() {
                error!("Child thread {:?} panicked: {:?}", id, error);
            }
        }

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let app = MyApp;
    app_main!(&app, log::LevelFilter::Info)?;

    Ok(())
}
