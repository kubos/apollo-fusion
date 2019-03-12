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

use failure::Error;
use kubos_app::*;

const EPS_TELEMETRY: &str = r#"{
		telemetry {
            lastEpsError { 
                daughterboard,
                motherboard,    
            },
            boardStatus {
                daughterboard, 
                motherboard, 
            },
            reset {
                automaticSoftware {
                    daughterboard,
                    motherboard
                },
                brownOut {
                    daughterboard,
                    motherboard
                },
                manual {
                    daughterboard,
                    motherboard
                },
                watchdog {
                    daughterboard,
                    motherboard
                }
            },
            motherboard {
                VoltageFeedingBcr1,
                CurrentBcr1Sa1a,
                CurrentBcr1Sa1b,
                ArrayTempSa1a,
                ArrayTempSa1b,
                SunDetectorSa1a,
                SunDetectorSa1b,
                VoltageFeedingBcr2,
                CurrentBcr2Sa2a,
                CurrentBcr2Sa2b,
                ArrayTempSa2a,
                ArrayTempSa2b,
                SunDetectorSa2a,
                SunDetectorSa2b,
                VoltageFeedingBcr3,
                CurrentBcr3Sa3a,
                CurrentBcr3Sa3b,
                ArrayTempSa3a,
                ArrayTempSa3b,
                SunDetectorSa3a,
                SunDetectorSa3b,
                BcrOutputCurrent,
                BcrOutputVoltage,
                CurrentDraw3V3,
                CurrentDraw5V,
                OutputCurrent12V,
                OutputVoltage12V,
                OutputCurrentBattery,
                OutputVoltageBattery,
                OutputCurrent5v,
                OutputVoltage5v,
                OutputCurrent33v,
                OutputVoltage33v,
                OutputVoltageSwitch1,
                OutputCurrentSwitch1,
                OutputVoltageSwitch2,
                OutputCurrentSwitch2,
                OutputVoltageSwitch3,
                OutputCurrentSwitch3,
                OutputVoltageSwitch4,
                OutputCurrentSwitch4,
                OutputVoltageSwitch5,
                OutputCurrentSwitch5,
                OutputVoltageSwitch6,
                OutputCurrentSwitch6,
                OutputVoltageSwitch7,
                OutputCurrentSwitch7,
                OutputVoltageSwitch8,
                OutputCurrentSwitch8,
                OutputVoltageSwitch9,
                OutputCurrentSwitch9,
                OutputVoltageSwitch10,
                OutputCurrentSwitch10,
                BoardTemperature,
            },
            daughterboard {
                VoltageFeedingBcr4,
                CurrentBcr4Sa4a,
                CurrentBcr4Sa4b,
                ArrayTempSa4a,
                ArrayTempSa4b,
                SunDetectorSa4a,
                SunDetectorSa4b,
                VoltageFeedingBcr5,
                CurrentBcr5Sa5a,
                CurrentBcr5Sa5b,
                ArrayTempSa5a,
                ArrayTempSa5b,
                SunDetectorSa5a,
                SunDetectorSa5b,
                VoltageFeedingBcr6,
                CurrentBcr6Sa6a,
                CurrentBcr6Sa6b,
                ArrayTempSa6a,
                ArrayTempSa6b,
                SunDetectorSa6a,
                SunDetectorSa6b,
                VoltageFeedingBcr7,
                CurrentBcr7Sa7a,
                CurrentBcr7Sa7b,
                ArrayTempSa7a,
                ArrayTempSa7b,
                SunDetectorSa7a,
                SunDetectorSa7b,
                VoltageFeedingBcr8,
                CurrentBcr8Sa8a,
                CurrentBcr8Sa8b,
                ArrayTempSa8a,
                ArrayTempSa8b,
                SunDetectorSa8a,
                SunDetectorSa8b,
                VoltageFeedingBcr9,
                CurrentBcr9Sa9a,
                CurrentBcr9Sa9b,
                ArrayTempSa9a,
                ArrayTempSa9b,
                SunDetectorSa9a,
                SunDetectorSa9b,
                BoardTemperature,
            }
        }"#;

pub fn get_telem() -> Result<(), Error> {
    let service = ServiceConfig::new("clyde-3g-eps-service");
    
    // Get all the basic telemetry
    let result = query(
        &service,
        EPS_TELEMETRY,
        Some(Duration::from_millis(100)
    ))?;
    
    let telemetry = &result["data"]["telemetry"];
    
    let telem_vec: Vec<(String, String)> = vec!();
    
    let last_error = &telemetry["lastEpsError"];
    
    telem_vec.push(("last_error_mb".to_owned(), last_error["motherboard"].as_str().unwrap_or("")));
    telem_vec.push(("last_error_db".to_owned(), last_error["daughterboard"].as_str().unwrap_or("")));
    
    let board_status = &telemetry["boardStatus"];
    
    telem_vec.push(("board_status_mb".to_owned(), last_error["motherboard"].as_str().unwrap_or("")));
    telem_vec.push(("board_status_db".to_owned(), last_error["daughterboard"].as_str().unwrap_or("")));
    
    let reset = &telemetry["reset"];
    
    telem_vec.push(("reset_sw_mb".to_owned(), reset["automaticSoftware"]["motherboard"].as_str().unwrap_or("")));
    telem_vec.push(("reset_sw_db".to_owned(), reset["automaticSoftware"]["daughterboard"].as_str().unwrap_or("")));
    
    telem_vec.push(("reset_brownout_mb".to_owned(), reset["brownOut"]["motherboard"].as_str().unwrap_or("")));
    telem_vec.push(("reset_brownout_db".to_owned(), reset["brownOut"]["daughterboard"].as_str().unwrap_or("")));
    
    telem_vec.push(("reset_manual_mb".to_owned(), reset["manual"]["motherboard"].as_str().unwrap_or("")));
    telem_vec.push(("reset_manual_db".to_owned(), reset["manual"]["daughterboard"].as_str().unwrap_or("")));
    
    telem_vec.push(("reset_wd_mb".to_owned(), reset["watchdog"]["motherboard"].as_str().unwrap_or("")));
    telem_vec.push(("reset_wd_db".to_owned(), reset["watchdog"]["daughterboard"].as_str().unwrap_or("")));
    
    let mb_telem = &telemetry["motherboard"].as_object();
    if let Some(data) = mb_telem {
        process_json(&mut telem_vec, data, "mb_".to_owned());
    }
    
    let db_telem = &telemetry["daughterboard"].as_object();
    if let Some(data) = mb_telem {
        process_json(&mut telem_vec, data, "db_".to_owned());
    }
    
    // Send all the telemetry to the telemetry database
    send_telem("EPS", telem_vec);
    
    Ok(())
}