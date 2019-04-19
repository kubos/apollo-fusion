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
// Message layout (20 bytes):
//  0: EPS motherboard tempurature
//  1: EPS daughterboard
//  2: EPS BCR 2 Side A
//  3: EPS BCR 2 Side B
//  4: EPS BCR 8 Side A
//  5: EPS BCR 8 Side B
//  6: EPS BCR 9 Side A
//  7: EPS BCR 9 Side B
//  8: MAI-400 gyroscope
//  9: MAI-400 RWS motor
// 10: BIM temperature sensor 1
// 11: BIM temperature sensor 2
// 12: BIM temperature sensor 3
// 13: BIM temperature sensor 4
// 14: BIM temperature sensor 5
// 15: BIM temperature sensor 6
// 16: BM2 internal temperature sensor
// 17: BM2 external temperature sensor 1 (TS1)
// 18: BM2 external temperature sensor 2 (TS2)
// 19: BM2 temperature range bit field - See section B.30 of the bq34z653 Technical Reference for details

// BM2 temperature range bit field
// 01: Temp < JT1 (below minimum operating temperature)
// 02: JT1  < Temp < JT2  (low, but okay temperature)
// 04: JT2  < Temp < JT2a (nominal temperature range 1)
// 08: JT2a < Temp < JT3  (nominal temperature range 2
// 10: JT3  < Temp < JT4  (high, but okay temperature)
// 20: JT4  < Temp (above maximum operating temperature)

use super::get_string;
use crate::transmit::*;
use failure::format_err;
use kubos_app::{query, ServiceConfig};
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

const EPS_BCR2A_TEMP: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_ArrayTempSa2a", limit: 1) {
        value
    }
}"#;

const EPS_BCR2B_TEMP: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_ArrayTempSa2b", limit: 1) {
        value
    }
}"#;

const EPS_BCR8A_TEMP: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_ArrayTempSa8a", limit: 1) {
        value
    }
}"#;

const EPS_BCR8B_TEMP: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_ArrayTempSa8b", limit: 1) {
        value
    }
}"#;

const EPS_BCR9A_TEMP: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_ArrayTempSa9a", limit: 1) {
        value
    }
}"#;

const EPS_BCR9B_TEMP: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_ArrayTempSa9b", limit: 1) {
        value
    }
}"#;

const MAI_GYRO_TEMP: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "rawImu_gyroTemp", limit: 1) {
        value
    }
}"#;

const MAI_MOTOR_TEMP: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "rwsMotorTemp", limit: 1) {
        value
    }
}"#;

const BIM_SENSOR_POWER: &str = r#"
    mutation {
        passthrough(module: "bim", command: "BIM:TEMP:POW ON") {
            status,
            command
        }
    }
"#;

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

const BM2_TEMP_RANGE: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "temp_range", limit: 1) {
        value
    }
}"#;

pub fn temp_packet(radios: Radios) {
    // Turn on the BIM's temperature sensors
    let service = ServiceConfig::new("pumpkin-mcu-service");

    let bim_sensors = query(&service, BIM_SENSOR_POWER, Some(Duration::from_millis(500))).is_ok();

    loop {
        // Float, *C. Operating temp -40 - 100*C, so can be represented by signed byte (i8)
        // Note: BCR 1, 2, 6, 7, 8, and 9 are connected, but only 2, 8, and 9 have temperature
        // sensors available
        let eps_mb_temp: u8 = (get_string(&radios, EPS_MB_TEMP)
            .parse::<f64>()
            .unwrap_or(0.0) as i8) as u8;
        let eps_db_temp: u8 = (get_string(&radios, EPS_DB_TEMP)
            .parse::<f64>()
            .unwrap_or(0.0) as i8) as u8;
        let eps_bcr2a_temp: u8 = (get_string(&radios, EPS_BCR2A_TEMP)
            .parse::<f64>()
            .unwrap_or(0.0) as i8) as u8;
        let eps_bcr2b_temp: u8 = (get_string(&radios, EPS_BCR2B_TEMP)
            .parse::<f64>()
            .unwrap_or(0.0) as i8) as u8;
        let eps_bcr8a_temp: u8 = (get_string(&radios, EPS_BCR8A_TEMP)
            .parse::<f64>()
            .unwrap_or(0.0) as i8) as u8;
        let eps_bcr8b_temp: u8 = (get_string(&radios, EPS_BCR8B_TEMP)
            .parse::<f64>()
            .unwrap_or(0.0) as i8) as u8;
        let eps_bcr9a_temp: u8 = (get_string(&radios, EPS_BCR9A_TEMP)
            .parse::<f64>()
            .unwrap_or(0.0) as i8) as u8;
        let eps_bcr9b_temp: u8 = (get_string(&radios, EPS_BCR9B_TEMP)
            .parse::<f64>()
            .unwrap_or(0.0) as i8) as u8;

        // No conversion needed. Raw value is *C, u8
        let mai_gyro_temp: u8 = get_string(&radios, MAI_GYRO_TEMP).parse().unwrap_or(0);
        // Temperature *C = gs_rwsMotorTemp * 0.0402930 - 50
        let mai_motor_temp: u8 = if let Ok(raw) = get_string(&radios, MAI_MOTOR_TEMP).parse::<i16>()
        {
            ((f32::from(raw) * 0.0402930 - 50.0) as i8) as u8
        } else {
            0
        };

        // Float, *K. Convert to *C
        // Setting the default values to `273.15` so that the resulting value is zero if we can't
        // get a good value
        let (bim_temp0, bim_temp1, bim_temp2, bim_temp3, bim_temp4, bim_temp5) = if bim_sensors {
            let temp0: u8 = ((get_string(&radios, BIM_TEMP0)
                .parse::<f64>()
                .unwrap_or(273.15)
                - 273.15) as i8) as u8;
            let temp1: u8 = ((get_string(&radios, BIM_TEMP1)
                .parse::<f64>()
                .unwrap_or(273.15)
                - 273.15) as i8) as u8;
            let temp2: u8 = ((get_string(&radios, BIM_TEMP2)
                .parse::<f64>()
                .unwrap_or(273.15)
                - 273.15) as i8) as u8;
            let temp3: u8 = ((get_string(&radios, BIM_TEMP3)
                .parse::<f64>()
                .unwrap_or(273.15)
                - 273.15) as i8) as u8;
            let temp4: u8 = ((get_string(&radios, BIM_TEMP4)
                .parse::<f64>()
                .unwrap_or(273.15)
                - 273.15) as i8) as u8;
            let temp5: u8 = ((get_string(&radios, BIM_TEMP5)
                .parse::<f64>()
                .unwrap_or(273.15)
                - 273.15) as i8) as u8;

            (temp0, temp1, temp2, temp3, temp4, temp5)
        } else {
            (0, 0, 0, 0, 0, 0)
        };

        // u16, 0.1*K. Convert to whole *C
        let raw: u16 = get_string(&radios, BM2_TEMP).parse().unwrap_or(2730);
        let bm2_temp: u8 = (raw / 10 - 273) as u8;
        // i16, 0.1*C. Convert to whole *C
        let raw: i16 = get_string(&radios, BM2_TS1_TEMP).parse().unwrap_or(0);
        let bm2_ts1_temp: u8 = (raw / 10) as u8;
        // i16, 0.1*C. Convert to whole *C
        let raw: i16 = get_string(&radios, BM2_TS2_TEMP).parse().unwrap_or(0);
        let bm2_ts2_temp: u8 = (raw / 10) as u8;
        // Temperature range bit field
        let raw: u16 = get_string(&radios, BM2_TEMP_RANGE).parse().unwrap_or(0);
        // Pulling out only the actual temp range bits
        let bm2_temp_range = (raw & 0x003F) as u8;

        // Turn into data packet
        let msg: [u8; 20] = [
            eps_mb_temp,
            eps_db_temp,
            eps_bcr2a_temp,
            eps_bcr2b_temp,
            eps_bcr8a_temp,
            eps_bcr8b_temp,
            eps_bcr9a_temp,
            eps_bcr9b_temp,
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
            bm2_temp_range,
        ];

        let _ = radios.transmit(MessageType::Temperature, 0, &msg);

        // Run every 15 minutes
        thread::sleep(Duration::from_secs(15 * 60));
    }
}
