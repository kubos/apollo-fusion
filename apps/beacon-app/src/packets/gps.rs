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

// Gather GPS telemetry every hour
//
// Note: All multi-byte fields are Little Endian
//
// Packet 1 (Position data. 27 bytes)
//     0: Position solution status (see `convert_solution_status`)
//   1-2: Position solution type (see `convert_posvel_type`)
//  3-10: (double) Position on x-axis
// 11-18: (double) Position on y-axis
// 18-26: (double) Position on z-axis
//
// Packet 2 (Velocity data. 27 bytes)
//     0: Velocity solution status (see `convert_solution_status`)
//   1-2: Velocity solution type (see `convert_posvel_type`)
//  3-10: (double) Velocity on x-axis
// 11-18: (double) Velocity on y-axis
// 18-26: (double) Velocity on z-axis
//
// Packet 3 (Everything else. 24 bytes)
//     0: Time status (how well the time is known. See `convert_time_status`)
//   1-2: Last known GPS time - Whole weeks since GPS epoch (Jan 6th, 1980)
//   3-7: Last known GPS time - Milliseconds elapsed in current week
//  8-11: System status flags  (see `convert_system_status`)
// 12-13: GPS status from AIM2
//    14: Power status from AIM2
// 15-16: Power draw over the 3.3V USB connection (normal value is ~0.9 Watts)
//    17: Power status from OEM7 service (0 = off, 1 = on)
// 18-19: Time from last successful lock - Whole weeks since GPS epoch (Jan 6th, 1980)
// 20-23: Time from last successful lock - Milliseconds elapsed in current week

// GPS status flags from AIM2 (Note: The returned value is 2 bytes, but there's only one useful
// byte of data):
// 		- 0x01 - Power is applied to OEM7
//      - 0x02 - RESET signal is applied to OEM7
//		- 0x04 - Position Valid pin on OEM7 is HIGH (On)
//      - 0x08 - UART passthrough to OEM7 is enabled

// Power status flags from AIM2:
// 		- 0x01 - Current is flowing from Vcc (should always be zero for us)
//      - 0x02 - Current is flowing from 3.3V USB
//      - 0x04 - Over-current/thermal fault on Vcc supply (should always be zero for us)
//      - 0x08 - Over-current/thermal fault on 3.3V USB supply

use super::get_string;
use crate::transmit::*;
use byteorder::{LittleEndian, WriteBytesExt};
use kubos_app::{query, ServiceConfig};
use std::thread;
use std::time::Duration;

const LOCKINFO_POS_X: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockInfo_position_0", limit: 1) {
        value
    }
}"#;

const LOCKINFO_POS_Y: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockInfo_position_1", limit: 1) {
        value
    }
}"#;

const LOCKINFO_POS_Z: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockInfo_position_2", limit: 1) {
        value
    }
}"#;

const LOCKINFO_VEL_X: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockInfo_velocity_0", limit: 1) {
        value
    }
}"#;

const LOCKINFO_VEL_Y: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockInfo_velocity_1", limit: 1) {
        value
    }
}"#;

const LOCKINFO_VEL_Z: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockInfo_velocity_2", limit: 1) {
        value
    }
}"#;

const LOCKINFO_TIME_MS: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockInfo_time_ms", limit: 1) {
        value
    }
}"#;

const LOCKINFO_TIME_WEEK: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockInfo_time_week", limit: 1) {
        value
    }
}"#;

const LOCKSTATUS_POS_STATUS: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockStatus_positionStatus", limit: 1) {
        value
    }
}"#;

const LOCKSTATUS_POS_TYPE: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockStatus_positionType", limit: 1) {
        value
    }
}"#;

const LOCKSTATUS_VEL_STATUS: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockStatus_velocityStatus", limit: 1) {
        value
    }
}"#;

const LOCKSTATUS_VEL_TYPE: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockStatus_velocityType", limit: 1) {
        value
    }
}"#;

const LOCKSTATUS_TIME_STATUS: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockStatus_timeStatus", limit: 1) {
        value
    }
}"#;

const LOCKSTATUS_TIME_MS: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockStatus_time_ms", limit: 1) {
        value
    }
}"#;

const LOCKSTATUS_TIME_WEEK: &str = r#"{
    telemetry(subsystem: "OEM", parameter: "lockStatus_time_week", limit: 1) {
        value
    }
}"#;

const SYSTEM_POWER: &str = r#"{
    power {
        uptime
    }
}"#;

const AIM2_GPS_STATUS: &str = r#"{
    telemetry(subsystem: "aim2", parameter: "status", limit: 1) {
        value
    }
}"#;

const AIM2_POWER_STATUS: &str = r#"{
    telemetry(subsystem: "aim2", parameter: "gps_power", limit: 1) {
        value
    }
}"#;

const AIM2_POWER_3V_USB: &str = r#"{
    telemetry(subsystem: "aim2", parameter: "oem_power2", limit: 1) {
        value
    }
}"#;

pub fn gps_packet(radios: Radios) {
    loop {
        send_position_packet(&radios);
        send_velocity_packet(&radios);
        send_misc_packet(&radios);

        // Run every hour
        thread::sleep(Duration::from_secs(3600));
    }
}

fn send_position_packet(radios: &Radios) {
    let position_status: u8 =
        convert_solution_status(&get_string(&radios, LOCKSTATUS_POS_STATUS).trim_matches('\"'));
    let position_type: u16 =
        convert_posvel_type(&get_string(&radios, LOCKSTATUS_POS_TYPE).trim_matches('\"'));

    let position_x: f64 = get_string(&radios, LOCKINFO_POS_X).parse().unwrap_or(0.0);
    let position_y: f64 = get_string(&radios, LOCKINFO_POS_Y).parse().unwrap_or(0.0);
    let position_z: f64 = get_string(&radios, LOCKINFO_POS_Z).parse().unwrap_or(0.0);

    let mut position_msg = vec![];
    position_msg.push(position_status);
    let _ = position_msg.write_u16::<LittleEndian>(position_type);
    let _ = position_msg.write_f64::<LittleEndian>(position_x);
    let _ = position_msg.write_f64::<LittleEndian>(position_y);
    let _ = position_msg.write_f64::<LittleEndian>(position_z);

    let _ = radios.transmit(MessageType::GPS, 1, &position_msg);
}

fn send_velocity_packet(radios: &Radios) {
    let velocity_status: u8 =
        convert_solution_status(&get_string(&radios, LOCKSTATUS_VEL_STATUS).trim_matches('\"'));
    let velocity_type: u16 =
        convert_posvel_type(&get_string(&radios, LOCKSTATUS_VEL_TYPE).trim_matches('\"'));

    let velocity_x: f64 = get_string(&radios, LOCKINFO_VEL_X).parse().unwrap_or(0.0);
    let velocity_y: f64 = get_string(&radios, LOCKINFO_VEL_Y).parse().unwrap_or(0.0);
    let velocity_z: f64 = get_string(&radios, LOCKINFO_VEL_Z).parse().unwrap_or(0.0);

    let mut velocity_msg = vec![];
    velocity_msg.push(velocity_status);
    let _ = velocity_msg.write_u16::<LittleEndian>(velocity_type);
    let _ = velocity_msg.write_f64::<LittleEndian>(velocity_x);
    let _ = velocity_msg.write_f64::<LittleEndian>(velocity_y);
    let _ = velocity_msg.write_f64::<LittleEndian>(velocity_z);

    let _ = radios.transmit(MessageType::GPS, 2, &velocity_msg);
}

fn send_misc_packet(radios: &Radios) {
    let time_status: u8 =
        convert_time_status(&get_string(&radios, LOCKSTATUS_TIME_STATUS).trim_matches('\"'));
    let time_week: u16 = get_string(&radios, LOCKSTATUS_TIME_WEEK)
        .parse()
        .unwrap_or(0);
    let time_ms: u32 = get_string(&radios, LOCKSTATUS_TIME_MS).parse().unwrap_or(0);

    let system_status: u32 = get_system_status(&radios);

    let gps_status: u16 =
        u16::from_str_radix(get_string(&radios, AIM2_GPS_STATUS).trim_matches('\"'), 16)
            .unwrap_or(0);
    let power_status: u8 = u8::from_str_radix(
        get_string(&radios, AIM2_POWER_STATUS).trim_matches('\"'),
        16,
    )
    .unwrap_or(0);
    let power_3v_usb: f32 = get_string(&radios, AIM2_POWER_3V_USB)
        .parse()
        .unwrap_or(0.0);

    let service = ServiceConfig::new("novatel-oem6-service");
    let power: u8 = match query(&service, SYSTEM_POWER, Some(Duration::from_millis(100))) {
        Ok(data) => {
            // Uptime will actually only ever be 0 (off) or 1 (on)
            data["power"]["uptime"].as_i64().unwrap_or(255) as u8
        }
        Err(_) => 255,
    };

    let lock_time_week: u16 = get_string(&radios, LOCKINFO_TIME_WEEK).parse().unwrap_or(0);
    let lock_time_ms: u32 = get_string(&radios, LOCKINFO_TIME_MS).parse().unwrap_or(0);

    let mut msg = vec![];
    msg.push(time_status);
    let _ = msg.write_u16::<LittleEndian>(time_week);
    let _ = msg.write_u32::<LittleEndian>(time_ms);
    let _ = msg.write_u32::<LittleEndian>(system_status);
    let _ = msg.write_u16::<LittleEndian>(gps_status);
    msg.push(power_status);
    let _ = msg.write_f32::<LittleEndian>(power_3v_usb);
    msg.push(power);
    let _ = msg.write_u16::<LittleEndian>(lock_time_week);
    let _ = msg.write_u32::<LittleEndian>(lock_time_ms);

    let _ = radios.transmit(MessageType::GPS, 3, &msg);
}

fn get_system_status(radios: &Radios) -> u32 {
    let request = r#"{
        telemetry(subsystem: "OEM", parameter: "systemStatus_status_0", limit: 1) {
            timestamp,
            value
        }
    }"#;

    let (benchmark, flag) = match query(
        &radios.telem_service,
        request,
        Some(Duration::from_millis(100)),
    ) {
        Ok(data) => {
            let flag = convert_system_status(data["telemetry"][0]["value"].as_str().unwrap_or(""));
            let timestamp = data["telemetry"][0]["timestamp"].as_f64().unwrap_or(0.0);
            (timestamp, flag)
        }
        Err(_) => (0.0, 0),
    };

    let mut flags: u32 = flag;

    for num in 1..24 {
        let request = format!(
            r#"{{
            telemetry(subsystem: "OEM", parameter: "systemStatus_status_{}", limit: 1) {{
                timestamp,
                value
            }}
        }}"#,
            num
        );

        let (timestamp, flag) = match query(
            &radios.telem_service,
            &request,
            Some(Duration::from_millis(100)),
        ) {
            Ok(data) => {
                let flag =
                    convert_system_status(data["telemetry"][0]["value"].as_str().unwrap_or(""));
                let timestamp = data["telemetry"][0]["timestamp"].as_f64().unwrap_or(0.0);
                (timestamp, flag)
            }
            Err(_) => (0.0, 0),
        };

        // We'll have a variable number of flags present, so we need to determine which ones are
        // only from the latest set of data.
        // - It should take less than two seconds to store all flags in the database
        // - If a flag index doesn't exist, its timestamp will be zero, resulting in a negative
        //   difference
        let diff = timestamp - benchmark;
        if diff > 2.0 || diff <= 0.0 {
            continue;
        }

        flags |= flag;
    }

    flags
}

fn convert_system_status(raw: &str) -> u32 {
    match raw.trim_matches('\"') {
        "ERROR_PRESENT" => 0x0000_0001,
        "TEMPERATURE_WARNING" => 0x0000_0002,
        "VOLTAGE_SUPPLY_WARNING" => 0x0000_0004,
        "ANTENNA_NOT_POWERED" => 0x0000_0008,
        "LNA_FAILURE" => 0x0000_0010,
        "ANTENNA_OPEN" => 0x0000_0020,
        "ANTENNA_SHORTENED" => 0x0000_0040,
        "CPU_OVERLOAD" => 0x0000_0080,
        "COM1_BUFFER_OVERRUN" => 0x0000_0100,
        "COM2_BUFFER_OVERRUN" => 0x0000_0200,
        "COM3_BUFFER_OVERRUN" => 0x0000_0400,
        "LINK_OVERRUN" => 0x0000_0800,
        "AUX_TRANSMIT_OVERRUN" => 0x0000_2000,
        "AGC_OUT_OF_RANGE" => 0x0000_4000,
        "INS_RESET" => 0x0001_0000,
        "GPS_ALMANAC_INVALID" => 0x0004_0000,
        "POSITION_SOLUTION_INVALID" => 0x0008_0000,
        "POSITION_FIXED" => 0x0010_0000,
        "CLOCK_STEERING_DISABLED" => 0x0020_0000,
        "CLOCK_MODEL_INVALID" => 0x0040_0000,
        "EXTERNAL_OSCILLATOR_LOCKED" => 0x0080_0000,
        "SOFTWARE_RESOURCE_WARNING" => 0x0100_0000,
        "AUX3_STATUS_EVENT" => 0x2000_0000,
        "AUX2_STATUS_EVENT" => 0x4000_0000,
        "AUX1_STATUS_EVENT" => 0x8000_0000,
        _ => 0,
    }
}

fn convert_solution_status(raw: &str) -> u8 {
    match raw.trim_matches('\"') {
        "SOL_COMPUTED" => 0,
        "INSUFFICIENT_OBSERVATIONS" => 1,
        "NO_CONVERGENCE" => 2,
        "SINGULARITY" => 3,
        "COVARIANCE_TRACE_EXCEEDED" => 4,
        "TEST_DISTANCE_EXCEEDED" => 5,
        "COLD_START" => 6,
        "HEIGHT_VELOCITY_EXCEEDED" => 7,
        "VARIANCE_EXCEEDED" => 8,
        "RESIDUALS_TOO_LARGE" => 9,
        "INTEGRITY_WARNING" => 13,
        "PENDING" => 18,
        "INVALID_FIX" => 19,
        "UNAUTHORIZED" => 20,
        _ => 255,
    }
}

fn convert_posvel_type(raw: &str) -> u16 {
    match raw.trim_matches('\"') {
        "NONE" => 0,
        "FIXED_POS" => 1,
        "FIXED_HEIGHT" => 2,
        "DOPPLER_VELOCITY" => 8,
        "SINGLE" => 16,
        "PSRDIFF" => 17,
        "WAAS" => 18,
        "PROPAGATED" => 19,
        "OMNISTAR" => 20,
        "L1FLOAT" => 32,
        "IONO_FREE_FLOAT" => 33,
        "NARROW_FLOAT" => 34,
        "L1INTEGER" => 48,
        "NARROW_INTEGER" => 50,
        "OMNISTAR_HP" => 64,
        "OMNISTAR_XP" => 65,
        "PPPCONVERGING" => 68,
        "PPP" => 69,
        "OPERATIONAL" => 70,
        "WARNING" => 71,
        "OUT_OF_BOUNDS" => 72,
        "PPPBASIC_CONVERGING" => 77,
        "PPPBASIC" => 78,
        _ => 0xFFFF,
    }
}

fn convert_time_status(raw: &str) -> u8 {
    match raw {
        "UNKNOWN" => 20,
        "APPROXIMATE" => 60,
        "COARSE_ADJUSTING" => 80,
        "COARSE" => 100,
        "COARSE_STEERING" => 120,
        "FREE_WHEELING" => 130,
        "FINE_ADJUSTING" => 140,
        "FINE" => 160,
        "FINE_BACKUP_STEERING" => 170,
        "FINE_STEERING" => 180,
        "SAT_TIME" => 200,
        _ => 255,
    }
}
