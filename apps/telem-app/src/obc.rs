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

// Gather telemetry from the MBM2 itself

use crate::telem_db::{process_json, send_telem};
use failure::Error;
use kubos_app::*;
use std::time::Duration;

const OBC_TELEMETRY: &str = r#"{
    memInfo {
        free,
        available
    }
}"#;

pub fn get_telem() -> Result<(), Error> {
    let service = ServiceConfig::new("monitor-service");

    let result = query(&service, OBC_TELEMETRY, Some(Duration::from_secs(1)))?;

    let mut telem_vec: Vec<(String, String)> = vec![];
    let telemetry = &result["memInfo"].as_object();

    // Auto-convert returned JSON into a flat key-value vector
    if let Some(data) = telemetry {
        process_json(&mut telem_vec, data, "memory_".to_owned());
    }

    // Send all the telemetry to the telemetry database
    send_telem("OBC", telem_vec);

    Ok(())
}
