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

// Gather telemetry from the NSL Duplex Radio

use crate::telem_db::{process_json, send_telem};
use failure::Error;
use kubos_app::*;
use std::time::Duration;

const DUPLEX_TELEMETRY: &str = r#"{
    packetsUp,
    failedPacketsUp,
    packetsDown,
    failedPacketsDown,
    modemHealth {
        resetCount,
        currentTime,
        currentRssi,
        connectionStatus,
        globalstarGateway,
        lastContactTime,
        lastAttemptTime,
        callAttemptsSinceReset,
        successfulConnectsSinceReset,
        averageConnectionDuration,
        connectionDurationStdDev
    },
    fileQueueCount,
    alive
}"#;

pub fn get_telem() -> Result<(), Error> {
    let service = ServiceConfig::new("nsl-duplex-d2-comms-service");

    let result = query(&service, DUPLEX_TELEMETRY, Some(Duration::from_secs(10)))?;

    let mut telem_vec: Vec<(String, String)> = vec![];

    // Flatten all of the telemetry
    if let Some(data) = &result.as_object() {
        process_json(&mut telem_vec, data, "".to_owned());
    }

    // Send all the telemetry to the telemetry database
    send_telem("DUPLEX", telem_vec);

    Ok(())
}
