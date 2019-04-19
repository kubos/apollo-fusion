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

// Gather RAM and storage space information every hour
//
// Message layout:
// 0: % RAM available
// 1: % user data partition in use (/home)
// 2: Deployment status (0 = not deployed, 1 = deployed)
// 3-33: free

use crate::transmit::*;
use kubos_app::query;
use kubos_system::UBootVars;
use log::*;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const STORAGE_QUERY: &str = r#"{
    telemetry(subsystem: "OBC", parameter: "memory_available", limit: 1) {
        timestamp,
        value
    }
}"#;

// Taken from /proc/meminfo on a BBB
const MEM_TOTAL: f32 = 515340.0;

pub fn obc_packet(radios: Radios) {
    let mut last_timestamp: String = "".to_string();

    loop {
        // Get last known memory values from telem db
        let ram_percent = match query(
            &radios.telem_service,
            STORAGE_QUERY,
            Some(Duration::from_millis(100)),
        ) {
            Ok(data) => {
                let mem: f32 = data["telemetry"][0]["value"]
                    .as_str()
                    .and_then(|val| val.parse().ok())
                    .unwrap_or(MEM_TOTAL);

                // Verify that this is a new value, not a repeat from the last time we asked
                let timestamp = data["telemetry"][0]["timestamp"].as_str().unwrap_or("");
                if timestamp == last_timestamp {
                    error!("Available memory timestamp has not changed");
                } else {
                    last_timestamp = timestamp.to_string();
                }

                // Convert to percentage, since that's a smaller number and basically what we care
                // about anyways
                let percent = (mem / MEM_TOTAL) * 100.0;

                percent
            }
            Err(error) => {
                error!("Unable to get last known memory usage: {:?}", error);
                0.0
            }
        };

        // Get the % of the user data partition that's free
        //
        // Since we're using the MBM2, we'll need to check both of the possible disk names.
        // If the SD card is present, then the eMMC will be mmc1. Otherwise, it will be mmc0.
        //
        // Note: I tried to just use a wildcard ("/dev/mmcblk*p4"), but couldn't get the correct
        // output for some reason, so we're doing this the long way.
        let disk_percent = if let Ok(output1) = Command::new("df").arg("/dev/mmcblk1p4").output() {
            let stdout = if output1.stderr.is_empty() {
                output1.stdout
            } else {
                if let Ok(output0) = Command::new("df").arg("/dev/mmcblk0p4").output() {
                    if output0.stderr.is_empty() {
                        output0.stdout
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
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
        
        let deployed = UBootVars::new().get_bool("deployed").unwrap_or(false);

        // Turn into data packet
        let msg: [u8; 3] = [
            ram_percent as u8,
            disk_percent,
            deployed as u8
        ];

        let _ = radios.transmit(MessageType::OBC, 0, &msg);

        thread::sleep(Duration::from_secs(3600));
    }
}
