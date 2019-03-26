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

// Message layout:
// 0: % RAM available
// 1-33: free

/*
OBC packet @ 1hr
- Meminfo
    - Storage
    - Ram
TODO: What’s useful for diagnosing OS issues?

*/

use crate::transmit::*;
use kubos_app::query;
use log::*;
use std::thread;
use std::time::Duration;

const STORAGE_QUERY: &str = r#"{
    telemetry(subsystem: "OBC", parameter: "memory_available", limit: 1) {
        timestamp,
        value
    }
}"#;

// TODO: Verify against AF stack, or change to be looked up at app start
// Taken from /proc/meminfo on a BBB
const MEM_TOTAL: f32 = 515340.0;

pub fn obc_packet(radios: Radios) {
    let mut last_timestamp: String = "".to_string();

    loop {
        // Get last known memory values from telem db
        match query(
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
                // Turn into data packet
                // TODO: Add disk space available
                let msg: [u8; 1] = [percent as u8];

                let _ = radios.transmit(MessageType::OBC, 0, &msg);
            }
            Err(error) => {
                error!("Unable to get last known memory usage");
            }
        }

        thread::sleep(Duration::from_secs(3600));
    }
}
