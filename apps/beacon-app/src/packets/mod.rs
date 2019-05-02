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

pub mod adcs;
pub mod errors;
pub mod gps;
pub mod obc;
pub mod power;
pub mod radio;
pub mod supmcu;
pub mod temperature;

use crate::transmit::*;
use kubos_app::query;
use std::time::Duration;

// Common function for reading an entry from the telemetry database
fn get_string(radios: &Radios, msg: &str) -> String {
    match query(&radios.telem_service, msg, Some(Duration::from_millis(100))) {
        Ok(data) => {
            let value = data["telemetry"][0]["value"].as_str().unwrap_or("");
            println!("Received: {}", value);
            value.to_owned()
        }
        Err(_) => "".to_owned(),
    }
}
