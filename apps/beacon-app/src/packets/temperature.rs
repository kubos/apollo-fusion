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

// Gather all available temperature readings every 15 minutes
//
// Message layout:
//  0: EPS motherboard tempurature
//  1: EPS daughterboard
//  2: MAI-400 gyroscope
//  3: MAI-400 RWS motor
//  4: BIM temperature sensor 1
//  5: BIM temperature sensor 2
//  6: BIM temperature sensor 3
//  7: BIM temperature sensor 4
//  8: BIM temperature sensor 5
//  9: BIM temperature sensor 6
// 10: BM2 internal temperature sensor
// 11: BM2 external temperature sensor 1 (TS1)
// 12: BM2 external temperature sensor 2 (TS2)

use crate::transmit::*;
use failure::format_err;
use kubos_app::query;
use log::*;
use std::thread;
use std::time::Duration;

const EPS_MB_TEMP: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_BoardTemperature", limit: 1) {
        value
    }
}"#;

const EPS_DB_TEMP: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_BoardTemperature", limit: 1) {
        value
    }
}"#;

const MAI_THERMOPILE_A_EARTHLIMB: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "irehs_thermopileStructA_earthLimb_temp", limit: 1) {
        value
    }
}"#;

const MAI_THERMOPILE_A_EARTHREF: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "irehs_thermopileStructA_earthRef_temp", limit: 1) {
        value
    }
}"#;

const MAI_THERMOPILE_A_SPACEREF: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "irehs_thermopileStructA_spaceRef_temp", limit: 1) {
        value
    }
}"#;

const MAI_THERMOPILE_A_WIDEFOV: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "irehs_thermopileStructA_wideFOV_temp", limit: 1) {
        value
    }
}"#;

const MAI_THERMOPILE_B_EARTHLIMB: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "irehs_thermopileStructB_earthLimb_temp", limit: 1) {
        value
    }
}"#;

const MAI_THERMOPILE_B_EARTHREF: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "irehs_thermopileStructB_earthRef_temp", limit: 1) {
        value
    }
}"#;

const MAI_THERMOPILE_B_SPACEREF: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "irehs_thermopileStructB_spaceRef_temp", limit: 1) {
        value
    }
}"#;

const MAI_THERMOPILE_B_WIDEFOV: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "irehs_thermopileStructB_wideFOV_temp", limit: 1) {
        value
    }
}"#;

const MAI_GYRO_TEMP: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "rawImu_gyroTemp", limit: 1) {
        value
    }
}"#;

const MAI_MOTOR_TEMP: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "rotating_rwsMotorTemp", limit: 1) {
        value
    }
}"#;

const BIM_TEMP0: &str = r#"{
    telemetry(subsystem: "bim", parameter: "temp0", limit: 1) {
        value
    }
}"#;

const BIM_TEMP1: &str = r#"{
    telemetry(subsystem: "bim", parameter: "temp1", limit: 1) {
        value
    }
}"#;

const BIM_TEMP2: &str = r#"{
    telemetry(subsystem: "bim", parameter: "temp2", limit: 1) {
        value
    }
}"#;

const BIM_TEMP3: &str = r#"{
    telemetry(subsystem: "bim", parameter: "temp3", limit: 1) {
        value
    }
}"#;

const BIM_TEMP4: &str = r#"{
    telemetry(subsystem: "bim", parameter: "temp4", limit: 1) {
        value
    }
}"#;

const BIM_TEMP5: &str = r#"{
    telemetry(subsystem: "bim", parameter: "temp5", limit: 1) {
        value
    }
}"#;

const BM2_TEMP: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "temperature", limit: 1) {
        value
    }
}"#;

// TODO: Maybe get/compare the extra BM2 temperature values

const BM2_TS1_TEMP: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "ts1_temp", limit: 1) {
        value
    }
}"#;

const BM2_TS2_TEMP: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "ts2_temp", limit: 1) {
        value
    }
}"#;

pub fn temp_packet(radios: Radios) {
    loop {
        // Operating temp -40 - 100*C, so can be represented by signed byte (i8)
        // Already converted from raw value
        let eps_mb_temp: u8 = get_string(&radios, EPS_MB_TEMP).parse::<i8>().unwrap_or(0) as u8;
        let eps_db_temp: u8 = get_string(&radios, EPS_DB_TEMP).parse::<i8>().unwrap_or(0) as u8;

        // TODO: Verify that a) we want/care about these values and
        // b) that the conversion equation is correct
        // 0.8059 mV per lsb
        // - There is likely an equation for converting from the voltage measured to the temperature
        //   measured
        let mai_thermopile_a_earthlimb = get_string(&radios, MAI_THERMOPILE_A_EARTHLIMB);
        let mai_thermopile_a_earthref = get_string(&radios, MAI_THERMOPILE_A_EARTHREF);
        let mai_thermopile_a_spaceref = get_string(&radios, MAI_THERMOPILE_A_SPACEREF);
        let mai_thermopile_a_widefov = get_string(&radios, MAI_THERMOPILE_A_WIDEFOV);
        let mai_thermopile_b_earthlimb = get_string(&radios, MAI_THERMOPILE_B_EARTHLIMB);
        let mai_thermopile_b_earthref = get_string(&radios, MAI_THERMOPILE_B_EARTHREF);
        let mai_thermopile_b_spaceref = get_string(&radios, MAI_THERMOPILE_B_SPACEREF);
        let mai_thermopile_b_widefov = get_string(&radios, MAI_THERMOPILE_B_WIDEFOV);
        // No conversion needed. Raw value is *C, u8
        let mai_gyro_temp: u8 = get_string(&radios, MAI_GYRO_TEMP).parse().unwrap_or(0);
        // Temperature *C = gs_rwsMotorTemp * 0.0402930 - 50
        // TODO : Figure out max value. Does it exceed 127?
        let raw: i16 = get_string(&radios, MAI_MOTOR_TEMP).parse().unwrap_or(0);
        let mai_motor_temp: u8 = (f32::from(raw) * 0.0402930 - 50.0) as u8;

        // Float, *K. Convert to *C
        // TODO (somewhere else): The BIM temperature sensors have to be turned on in order to
        // actually measure the temperature. `BIM:TEMP:POW ON`
        // TODO: These might be raw values that need to be adjusted with the scalar and offset?
        // See the supMCU manual...
        let bim_temp0: u8 =
            (get_string(&radios, BIM_TEMP0).parse::<f64>().unwrap_or(0.0) - 273.15) as u8;
        let bim_temp1: u8 =
            (get_string(&radios, BIM_TEMP1).parse::<f64>().unwrap_or(0.0) - 273.15) as u8;
        let bim_temp2: u8 =
            (get_string(&radios, BIM_TEMP2).parse::<f64>().unwrap_or(0.0) - 273.15) as u8;
        let bim_temp3: u8 =
            (get_string(&radios, BIM_TEMP3).parse::<f64>().unwrap_or(0.0) - 273.15) as u8;
        let bim_temp4: u8 =
            (get_string(&radios, BIM_TEMP4).parse::<f64>().unwrap_or(0.0) - 273.15) as u8;
        let bim_temp5: u8 =
            (get_string(&radios, BIM_TEMP5).parse::<f64>().unwrap_or(0.0) - 273.15) as u8;

        // u16, 0.1*K. Convert to whole *C
        let raw: u16 = get_string(&radios, BM2_TEMP).parse().unwrap_or(0);
        let bm2_temp: u8 = (raw / 10 - 273) as u8;
        // i16, 0.1*C. Convert to whole *C
        let raw: i16 = get_string(&radios, BM2_TS1_TEMP).parse().unwrap_or(0);
        let bm2_ts1_temp: u8 = (raw / 10) as u8;
        // i16, 0.1*C. Convert to whole *C
        let raw: i16 = get_string(&radios, BM2_TS2_TEMP).parse().unwrap_or(0);
        let bm2_ts2_temp: u8 = (raw / 10) as u8;

        // Turn into data packet
        let msg: [u8; 13] = [
            eps_mb_temp,
            eps_db_temp,
            mai_gyro_temp,
            mai_motor_temp,
            bim_temp0,
            bim_temp1,
            bim_temp2,
            bim_temp3,
            bim_temp4,
            bim_temp5,
            bm2_temp,
            bm2_ts1_temp,
            bm2_ts2_temp,
        ];

        let _ = radios.transmit(MessageType::Temperature, 0, &msg);

        // Run every 15 minutes
        thread::sleep(Duration::from_secs(15 * 60));
    }
}

fn get_string(radios: &Radios, msg: &str) -> String {
    match query(&radios.telem_service, msg, Some(Duration::from_millis(100))) {
        Ok(data) => {
            let value = data["telemetry"][0]["value"].as_str().unwrap_or("0");
            value.to_owned()
        }
        Err(_) => "0".to_owned(),
    }
}
