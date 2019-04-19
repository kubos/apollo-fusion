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

// Gather ADCS telemetry every hour
//
// Note: All multi-byte fields are Little Endian
//
// Packet 1 (18 bytes):
//   0-3: GPS time (UTC time in seconds)
//   4-5: # of good commands received
//   6-7: # of invalid commands received
//   8-9: # of commands received with bad checksums
//    10: Last command received
//    11: ACS Mode (see `convert_acs_mode`)
//    12: Attitude determination mode
//    13: Eclipse flag (0 = not eclipsed, 1 = eclipsed)
// 14-17: Angle to go ("Net angle required before target attitude is achieved")
//
// Packet 2 (32 bytes):
// 0-3: Body rate, x-axis
// 4-7: Body rate, y-axis
// 8-11: Body rate, z-axis
// 12-13: Wheel speed, x-axis
// 14-15: Wheel speed, y-axis
// 16-17: Wheel speed, z-axis
// 18-19: Wheel bias, x-axis
// 20-21: Wheel bias, y-axis
// 22-23: Wheel bias, z-axis
// 24-25: Current estimated orbit-to-body quaternion, param 0
// 26-27: Current estimated orbit-to-body quaternion, param 1
// 28-29: Current estimated orbit-to-body quaternion, param 2
// 30-31: Current estimated orbit-to-body quaternion, param 3

// Attitude determination modes:
// 0 - CSS/magnetometer
// 1 - Set Qbi
// 2 - EHS/magnetometer

use super::get_string;
use crate::transmit::*;
use byteorder::{LittleEndian, WriteBytesExt};
use failure::format_err;
use kubos_app::{query, ServiceConfig};
use log::*;
use std::thread;
use std::time::Duration;

const GPS_TIME: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "gpsTime", limit: 1) {
        value
    }
}"#;

const GOOD_CMD_COUNT: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "cmdValidCntr", limit: 1) {
        value
    }
}"#;

const BAD_CMD_COUNT: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "cmdInvalidCntr", limit: 1) {
        value
    }
}"#;

const BAD_CHECKSUM_COUNT: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "cmdInvalidChksumCntr", limit: 1) {
        value
    }
}"#;

const LAST_COMMAND: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "lastCommand", limit: 1) {
        value
    }
}"#;

const ACS_MODE: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "acsMode", limit: 1) {
        value
    }
}"#;

const ATTDET_MODE: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "attDetMode", limit: 1) {
        value
    }
}"#;

const ECLIPSE: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "eclipseFlag", limit: 1) {
        value
    }
}"#;

const ANGLE_TO_GO: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "angleToGo", limit: 1) {
        value
    }
}"#;

const BODY_RATE_X: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "omegaB_0", limit: 1) {
        value
    }
}"#;

const BODY_RATE_Y: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "omegaB_1", limit: 1) {
        value
    }
}"#;

const BODY_RATE_Z: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "omegaB_2", limit: 1) {
        value
    }
}"#;

const WHEEL_SPEED_X: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "rwsSpeedTach_0", limit: 1) {
        value
    }
}"#;

const WHEEL_SPEED_Y: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "rwsSpeedTach_1", limit: 1) {
        value
    }
}"#;

const WHEEL_SPEED_Z: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "rwsSpeedTach_2", limit: 1) {
        value
    }
}"#;

const WHEEL_BIAS_X: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "wheelSpeedBias_0", limit: 1) {
        value
    }
}"#;

const WHEEL_BIAS_Y: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "wheelSpeedBias_1", limit: 1) {
        value
    }
}"#;

const WHEEL_BIAS_Z: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "wheelSpeedBias_2", limit: 1) {
        value
    }
}"#;

const QBO_QUATERNION_0: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "qboHat_0", limit: 1) {
        value
    }
}"#;

const QBO_QUATERNION_1: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "qboHat_1", limit: 1) {
        value
    }
}"#;

const QBO_QUATERNION_2: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "qboHat_2", limit: 1) {
        value
    }
}"#;

const QBO_QUATERNION_3: &str = r#"{
    telemetry(subsystem: "MAI400", parameter: "qboHat_3", limit: 1) {
        value
    }
}"#;

pub fn adcs_packet(radios: Radios) {
    loop {
        // Packet 1
        let gps_time: u32 = get_string(&radios, GPS_TIME).parse().unwrap_or(0);

        let good_cmd_count: u16 = get_string(&radios, GOOD_CMD_COUNT).parse().unwrap_or(0);
        let bad_cmd_count: u16 = get_string(&radios, BAD_CMD_COUNT).parse().unwrap_or(0);
        let bad_checksum_count: u16 = get_string(&radios, BAD_CHECKSUM_COUNT).parse().unwrap_or(0);
        let last_command: u8 = get_string(&radios, LAST_COMMAND).parse().unwrap_or(0);

        let acs_mode: u8 = convert_acs_mode(&get_string(&radios, ACS_MODE));
        let attdet_mode: u8 = get_string(&radios, ATTDET_MODE).parse().unwrap_or(255);

        let eclipse: u8 = get_string(&radios, ECLIPSE).parse().unwrap_or(255);

        let angle_to_go: f32 = get_string(&radios, ANGLE_TO_GO).parse().unwrap_or(0.0);

        let mut msg = vec![];
        let _ = msg.write_u32::<LittleEndian>(gps_time);
        let _ = msg.write_u16::<LittleEndian>(good_cmd_count);
        let _ = msg.write_u16::<LittleEndian>(bad_cmd_count);
        let _ = msg.write_u16::<LittleEndian>(bad_checksum_count);
        msg.push(last_command);
        msg.push(acs_mode);
        msg.push(attdet_mode);
        msg.push(eclipse);
        let _ = msg.write_f32::<LittleEndian>(angle_to_go);

        let _ = radios.transmit(MessageType::ADCS, 1, &msg);

        // Packet 2
        let body_rate_x: f32 = get_string(&radios, BODY_RATE_X).parse().unwrap_or(0.0);
        let body_rate_y: f32 = get_string(&radios, BODY_RATE_Y).parse().unwrap_or(0.0);
        let body_rate_z: f32 = get_string(&radios, BODY_RATE_Z).parse().unwrap_or(0.0);

        let wheel_speed_x: i16 = get_string(&radios, WHEEL_SPEED_X).parse().unwrap_or(0);
        let wheel_speed_y: i16 = get_string(&radios, WHEEL_SPEED_Y).parse().unwrap_or(0);
        let wheel_speed_z: i16 = get_string(&radios, WHEEL_SPEED_Z).parse().unwrap_or(0);

        let wheel_bias_x: i16 = get_string(&radios, WHEEL_BIAS_X).parse().unwrap_or(0);
        let wheel_bias_y: i16 = get_string(&radios, WHEEL_BIAS_Y).parse().unwrap_or(0);
        let wheel_bias_z: i16 = get_string(&radios, WHEEL_BIAS_Z).parse().unwrap_or(0);

        let qbo_0: i16 = get_string(&radios, QBO_QUATERNION_0).parse().unwrap_or(0);
        let qbo_1: i16 = get_string(&radios, QBO_QUATERNION_1).parse().unwrap_or(0);
        let qbo_2: i16 = get_string(&radios, QBO_QUATERNION_2).parse().unwrap_or(0);
        let qbo_3: i16 = get_string(&radios, QBO_QUATERNION_3).parse().unwrap_or(0);

        let mut msg = vec![];
        let _ = msg.write_f32::<LittleEndian>(body_rate_x);
        let _ = msg.write_f32::<LittleEndian>(body_rate_y);
        let _ = msg.write_f32::<LittleEndian>(body_rate_z);
        let _ = msg.write_i16::<LittleEndian>(wheel_speed_x);
        let _ = msg.write_i16::<LittleEndian>(wheel_speed_y);
        let _ = msg.write_i16::<LittleEndian>(wheel_speed_z);
        let _ = msg.write_i16::<LittleEndian>(wheel_bias_x);
        let _ = msg.write_i16::<LittleEndian>(wheel_bias_y);
        let _ = msg.write_i16::<LittleEndian>(wheel_bias_z);
        let _ = msg.write_i16::<LittleEndian>(qbo_0);
        let _ = msg.write_i16::<LittleEndian>(qbo_1);
        let _ = msg.write_i16::<LittleEndian>(qbo_2);
        let _ = msg.write_i16::<LittleEndian>(qbo_3);

        let _ = radios.transmit(MessageType::ADCS, 2, &msg);

        // Run every hour
        thread::sleep(Duration::from_secs(3600));
    }
}

fn convert_acs_mode(raw: &str) -> u8 {
    match raw.trim_matches('\"') {
        "TEST_MODE" => 0,
        "RATE_NULLING" => 1,
        "RESERVED1" => 2,
        "NADIR_POINTING" => 3,
        "LAT_LONG_POINTING" => 4,
        "QBX_MODE" => 5,
        "RESERVED2" => 6,
        "NORMAL_SUN" => 7,
        "LAT_LONG_SUN" => 8,
        "QINTERTIAL" => 9,
        "RESERVED3" => 10,
        "QTABLE" => 11,
        "SUN_RAM" => 12,
        _ => 0xFF,
    }
}
