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

use crate::telem_db::send_telem;
use failure::Error;
use kubos_app::*;
use std::time::Duration;

pub fn get_telem() -> Result<(), Error> {
    let service = ServiceConfig::new("pumpkin-mcu-service");

    // Get all the telemetry for all the Sup MCU modules we have
    let modules = ["aim2", "bim", "pim", "rhm", "epsm"];

    for module in modules.iter() {
        let result = query(
            &service,
            &format!("{{mcuTelemetry(module: \"{}\")}}", module),
            // The delay is 200ms per field requested.
            // Each module will have ~20 fields
            Some(Duration::from_secs(10)),
        )?;

        let telem_raw = result["mcuTelemetry"].as_str().unwrap_or("");
        let telem: serde_json::Value = serde_json::from_str(telem_raw)?;

        let mut telem_vec: Vec<(String, String)> = vec![];
        if let Some(inner) = telem.as_object() {
            for (key, value) in inner.iter() {
                let data = &value["data"];

                telem_vec.push((key.to_owned(), format!("{}", data)));
            }
        }

        send_telem(module, telem_vec);
    }

    Ok(())
}
