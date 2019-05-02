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

// Health and Status Beacon Application
//
// Starts up threads for each of the beacon packets:
// - Power
// - Temperature
// - Errors
// - OBC
// - SupMCU
// - GPS
// - ADCS
// - Radio (duplex)

mod packets;
mod transmit;

use crate::packets::*;
use crate::transmit::*;
use failure::Error;
use kubos_app::*;
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct MyApp;

const THREAD_INTERVAL: Duration = Duration::from_secs(30);

impl AppHandler for MyApp {
    fn on_boot(&self, _args: Vec<String>) -> Result<(), Error> {
        // Beacon app will be started on-demand by the deployment app
        Ok(())
    }

    fn on_command(&self, _args: Vec<String>) -> Result<(), Error> {
        let mut handles = vec![];
        // We're using the RHM supMCU module's connection to the Simplex (over I2C),
        // rather than a direct UART connection
        let sup_mcu = ServiceConfig::new("pumpkin-mcu-service");
        let telem_service = ServiceConfig::new("telemetry-service");
        let radios = Radios {
            telem_service,
            simplex: Arc::new(Mutex::new(sup_mcu)),
        };

        // Spawn threads for each of the beacon messages
        // (putting a 30 second delay in between each one to help prevent them from running at
        // exactly the same time)

        let power_radios = radios.clone();
        let handle = thread::spawn(move || power::power_packet(power_radios));
        debug!("Spawning power beacon thread: {:?}", handle.thread().id());
        handles.push(handle);
        thread::sleep(THREAD_INTERVAL);

        let temp_radios = radios.clone();
        let handle = thread::spawn(move || temperature::temp_packet(temp_radios));
        debug!(
            "Spawning temperature beacon thread: {:?}",
            handle.thread().id()
        );
        handles.push(handle);
        thread::sleep(THREAD_INTERVAL);

        let errors_radios = radios.clone();
        let handle = thread::spawn(move || errors::errors_packet(errors_radios));
        debug!("Spawning errors beacon thread: {:?}", handle.thread().id());
        handles.push(handle);
        thread::sleep(THREAD_INTERVAL);

        let obc_radios = radios.clone();
        let handle = thread::spawn(move || obc::obc_packet(obc_radios));
        debug!("Spawning OBC beacon thread: {:?}", handle.thread().id());
        handles.push(handle);
        thread::sleep(THREAD_INTERVAL);

        let supmcu_radios = radios.clone();
        let handle = thread::spawn(move || supmcu::supmcu_packet(supmcu_radios));
        debug!("Spawning supMCU beacon thread: {:?}", handle.thread().id());
        handles.push(handle);
        thread::sleep(THREAD_INTERVAL);

        let gps_radios = radios.clone();
        let handle = thread::spawn(move || gps::gps_packet(gps_radios));
        debug!("Spawning GPS beacon thread: {:?}", handle.thread().id());
        handles.push(handle);
        thread::sleep(THREAD_INTERVAL);

        let adcs_radios = radios.clone();
        let handle = thread::spawn(move || adcs::adcs_packet(adcs_radios));
        debug!("Spawning ADCS beacon thread: {:?}", handle.thread().id());
        handles.push(handle);
        thread::sleep(THREAD_INTERVAL);

        // TODO: Radio (duplex) packet

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
