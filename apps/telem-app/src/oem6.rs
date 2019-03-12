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

use crate::telem_db::{process_json, send_telem};
use failure::Error;
use kubos_app::*;
use std::time::Duration;

// Note: Debug telemetry is omitted because it is only version/model info,
// which does not change
const OEM6_TELEMETRY: &str = r#"{
    errors,
    telemetry {
        nominal {
            lockInfo {
            	position,
            	time {
            		ms,
            		week
            	},
            	velocity
            },
            lockStatus {
            	positionStatus,
            	positionType,
            	time {
            		ms,
            		week
            	},
            	timeStatus,
            	velocityStatus,
            	velocityType
            },
            systemStatus {
            	errors,
            	status
            }
        }
    }
}"#;

const OEM6_POWER: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "GPS:POW ON") {
            status,
            command
        }
    }
"#;

const OEM6_COMM: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "GPS:COMM UART3") {
            status,
            command
        }
    }
"#;

const OEM6_PASS: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "GPS:PASS ON") {
            status,
            command
        }
    }
"#;

pub fn get_telem() -> Result<(), Error> {
    // Make sure the OEM6 is on and able to communicate with us
    let service = ServiceConfig::new("pumpkin-mcu-service");

    let _ = query(&service, OEM6_POWER, Some(Duration::from_millis(500)))?;

    let _ = query(&service, OEM6_COMM, Some(Duration::from_millis(500)))?;

    let _ = query(&service, OEM6_PASS, Some(Duration::from_millis(500)))?;

    let service = ServiceConfig::new("novatel-oem6-service");

    // Get all the basic telemetry
    let result = query(&service, OEM6_TELEMETRY, Some(Duration::from_secs(1)))?;

    let mut telem_vec: Vec<(String, String)> = vec![];
    let nominal = &result["telemetry"]["nominal"].as_object();

    // Auto-convert returned JSON into a flat key-value vector
    if let Some(data) = nominal {
        process_json(&mut telem_vec, data, "".to_owned());
    }

    // Send it to the telemetry database
    send_telem("OEM6", telem_vec);

    Ok(())
}
