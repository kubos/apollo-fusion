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

// Gather telemetry from the EPS and batteries every 15 minutes
//
// Note: All multi-byte fields are Little Endian
//
// Packet 1 (General status info. 32 bytes)
//   0-1: Battery pack voltage from BM2
//   2-3: Battery pack current from BM2
//   4-5: Permanent failure status flags (see B.7 of the bq34z653 technical reference doc)
//     6: Motherboard brown-out reset count
//     7: Motherboard WDT reset count
//     8: Motherboard automatic software reset count
//     9: Daughterboard brown-out reset count
//    10: Daughterboard WDT reset count
//    11: Daughterboard automatic software reset count
// 12-13: Remaining capacity (mA)
// 14-15: Full capacity (mA)
// 16-17: Charging voltage (mV)
// 18-19: Charging current (mA)
// 20-21: Output voltage of EPS' 12V bus (mV)
// 22-23: Output current of EPS' 12V bus (mA)
// 24-25: Output voltage of EPS' 5V bus (mV)
// 26-27: Output current of EPS' 5V bus (mA)
// 28-29: Output voltage of EPS' 3.3V bus (mV)
// 30-31: Output current of EPS' 3.3V bus (mA)
//
// Packet 2 (Battery cells + motherboard solar panels. 20 bytes)
//   0-1: Battery cell 1 voltage (mV)
//   2-3: Battery cell 2 voltage (mV)
//   4-5: Battery cell 3 voltage (mV)
//   6-7: Battery cell 4 voltage (mV)
//   8-9: BCR 1 voltage (mV)
// 10-11: BCR 1 connector A current (mA)
// 12-13: BCR 1 connector B current (mA)
// 14-15: BCR 2 voltage (mV)
// 16-17: BCR 2 connector A current (mA)
// 18-19: BCR 2 connector B current (mA)
//
// Packet 3 (Daughterboard solar panels. 24 bytes)
//   0-1: BCR 6 voltage (mV)
//   2-3: BCR 6 connector A current (mA)
//   4-5: BCR 6 connector B current (mA)
//   6-7: BCR 7 voltage (mV)
//   8-9: BCR 7 connector A current (mA)
// 10-11: BCR 7 connector B current (mA)
// 12-13: BCR 8 voltage (mV)
// 14-15: BCR 8 connector A current (mA)
// 16-17: BCR 8 connector B current (mA)
// 18-19: BCR 9 voltage (mV)
// 20-21: BCR 9 connector A current (mA)
// 22-23: BCR 9 connector B current (mA)

use super::get_string;
use crate::transmit::*;
use byteorder::{LittleEndian, WriteBytesExt};
use failure::format_err;
use kubos_app::{query, ServiceConfig};
use log::*;
use std::thread;
use std::time::Duration;

const VOLTAGE: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "voltage", limit: 1) {
        value
    }
}"#;

const CURRENT: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "current", limit: 1) {
        value
    }
}"#;

const PF_STATUS: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "perm_fail_status", limit: 1) {
        value
    }
}"#;

const MB_RESET_BO: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "reset_brownout_mb", limit: 1) {
        value
    }
}"#;

const MB_RESET_WDT: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "reset_wd_mb", limit: 1) {
        value
    }
}"#;

const MB_RESET_SW: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "reset_sw_mb", limit: 1) {
        value
    }
}"#;

const DB_RESET_BO: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "reset_brownout_mb", limit: 1) {
        value
    }
}"#;

const DB_RESET_WDT: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "reset_wd_mb", limit: 1) {
        value
    }
}"#;

const DB_RESET_SW: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "reset_sw_mb", limit: 1) {
        value
    }
}"#;

const REMAINING_CAPACITY: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "remaining_capacity", limit: 1) {
        value
    }
}"#;

const FULL_CAPACITY: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "full_capacity", limit: 1) {
        value
    }
}"#;

const CHARGE_VOLTAGE: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "charging_voltage", limit: 1) {
        value
    }
}"#;

const CHARGE_CURRENT: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "charging_current", limit: 1) {
        value
    }
}"#;

const VOLTAGE_12V: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_OutputVoltage12V", limit: 1) {
        value
    }
}"#;

const CURRENT_12V: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_OutputCurrent12V", limit: 1) {
        value
    }
}"#;

const VOLTAGE_5V: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_OutputVoltage5v", limit: 1) {
        value
    }
}"#;

const CURRENT_5V: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_OutputCurrent5v", limit: 1) {
        value
    }
}"#;

const VOLTAGE_3V: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_OutputVoltage33v", limit: 1) {
        value
    }
}"#;

const CURRENT_3V: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_OutputCurrent33v", limit: 1) {
        value
    }
}"#;

const VOLTAGE_CELL1: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "cell1_voltage", limit: 1) {
        value
    }
}"#;

const VOLTAGE_CELL2: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "cell2_voltage", limit: 1) {
        value
    }
}"#;

const VOLTAGE_CELL3: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "cell3_voltage", limit: 1) {
        value
    }
}"#;

const VOLTAGE_CELL4: &str = r#"{
    telemetry(subsystem: "bm2", parameter: "cell4_voltage", limit: 1) {
        value
    }
}"#;

const VOLTAGE_BCR1: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_VoltageFeedingBcr1", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR1_A: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_CurrentBcr1Sa1a", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR1_B: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_CurrentBcr1Sa1b", limit: 1) {
        value
    }
}"#;

const VOLTAGE_BCR2: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_VoltageFeedingBcr2", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR2_A: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_CurrentBcr2Sa2a", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR2_B: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "mb_CurrentBcr2Sa2b", limit: 1) {
        value
    }
}"#;

const VOLTAGE_BCR6: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_VoltageFeedingBcr6", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR6_A: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_CurrentBcr6Sa6a", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR6_B: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_CurrentBcr6Sa6b", limit: 1) {
        value
    }
}"#;

const VOLTAGE_BCR7: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_VoltageFeedingBcr7", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR7_A: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_CurrentBcr7Sa7a", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR7_B: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_CurrentBcr7Sa7b", limit: 1) {
        value
    }
}"#;

const VOLTAGE_BCR8: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_VoltageFeedingBcr8", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR8_A: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_CurrentBcr8Sa8a", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR8_B: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_CurrentBcr8Sa8b", limit: 1) {
        value
    }
}"#;

const VOLTAGE_BCR9: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_VoltageFeedingBcr9", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR9_A: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_CurrentBcr9Sa9a", limit: 1) {
        value
    }
}"#;

const CURRENT_BCR9_B: &str = r#"{
    telemetry(subsystem: "EPS", parameter: "db_CurrentBcr9Sa9b", limit: 1) {
        value
    }
}"#;

pub fn power_packet(radios: Radios) {
    loop {
        let voltage: u16 = get_string(&radios, VOLTAGE).parse().unwrap_or(0xFFFF);
        let current: i16 = get_string(&radios, CURRENT).parse().unwrap_or(0x7FFF);

        let pf_status: u16 =
            u16::from_str_radix(get_string(&radios, PF_STATUS).trim_matches('\"'), 16)
                .unwrap_or(0xFFFF);

        let mb_reset_bo: u8 = get_string(&radios, MB_RESET_BO).parse().unwrap_or(0xFF);
        let mb_reset_wdt: u8 = get_string(&radios, MB_RESET_WDT).parse().unwrap_or(0xFF);
        let mb_reset_sw: u8 = get_string(&radios, MB_RESET_SW).parse().unwrap_or(0xFF);

        let db_reset_bo: u8 = get_string(&radios, DB_RESET_BO).parse().unwrap_or(0xFF);
        let db_reset_wdt: u8 = get_string(&radios, DB_RESET_WDT).parse().unwrap_or(0xFF);
        let db_reset_sw: u8 = get_string(&radios, DB_RESET_SW).parse().unwrap_or(0xFF);

        let remaining_cap: u16 = get_string(&radios, REMAINING_CAPACITY)
            .parse()
            .unwrap_or(0xFFFF);
        let full_cap: u16 = get_string(&radios, FULL_CAPACITY).parse().unwrap_or(0xFFFF);

        let charge_voltage: u16 = get_string(&radios, CHARGE_VOLTAGE)
            .parse()
            .unwrap_or(0xFFFF);
        let charge_current: u16 = get_string(&radios, CHARGE_CURRENT)
            .parse()
            .unwrap_or(0xFFFF);

        // Convert voltages from f64 V to i16 mV
        // Convert currents from f64 mA to i16 mA
        let voltage_12v: i16 = (get_string(&radios, VOLTAGE_12V)
            .parse::<f64>()
            .unwrap_or(0.0)
            * 1000.0) as i16;
        let current_12v: i16 = get_string(&radios, CURRENT_12V)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;
        let voltage_5v: i16 = (get_string(&radios, VOLTAGE_5V)
            .parse::<f64>()
            .unwrap_or(0.0)
            * 1000.0) as i16;
        let current_5v: i16 = get_string(&radios, CURRENT_5V)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;
        let voltage_3v: i16 = (get_string(&radios, VOLTAGE_3V)
            .parse::<f64>()
            .unwrap_or(0.0)
            * 1000.0) as i16;
        let current_3v: i16 = get_string(&radios, CURRENT_3V)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;

        let mut msg = vec![];
        let _ = msg.write_u16::<LittleEndian>(voltage);
        let _ = msg.write_i16::<LittleEndian>(current);
        let _ = msg.write_u16::<LittleEndian>(pf_status);
        msg.push(mb_reset_bo);
        msg.push(mb_reset_wdt);
        msg.push(mb_reset_sw);
        msg.push(db_reset_bo);
        msg.push(db_reset_wdt);
        msg.push(db_reset_sw);
        let _ = msg.write_u16::<LittleEndian>(remaining_cap);
        let _ = msg.write_u16::<LittleEndian>(full_cap);
        let _ = msg.write_u16::<LittleEndian>(charge_voltage);
        let _ = msg.write_u16::<LittleEndian>(charge_current);
        let _ = msg.write_i16::<LittleEndian>(voltage_12v);
        let _ = msg.write_i16::<LittleEndian>(current_12v);
        let _ = msg.write_i16::<LittleEndian>(voltage_5v);
        let _ = msg.write_i16::<LittleEndian>(current_5v);
        let _ = msg.write_i16::<LittleEndian>(voltage_3v);
        let _ = msg.write_i16::<LittleEndian>(current_3v);

        let _ = radios.transmit(MessageType::Power, 1, &msg);

        let voltage_cell1: u16 = get_string(&radios, VOLTAGE_CELL1).parse().unwrap_or(0xFFFF);
        let voltage_cell2: u16 = get_string(&radios, VOLTAGE_CELL2).parse().unwrap_or(0xFFFF);
        let voltage_cell3: u16 = get_string(&radios, VOLTAGE_CELL3).parse().unwrap_or(0xFFFF);
        let voltage_cell4: u16 = get_string(&radios, VOLTAGE_CELL4).parse().unwrap_or(0xFFFF);

        let voltage_bcr1: i16 = (get_string(&radios, VOLTAGE_BCR1)
            .parse::<f64>()
            .unwrap_or(0.0)
            * 1000.0) as i16;
        let current_bcr1a: i16 = get_string(&radios, CURRENT_BCR1_A)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;
        let current_bcr1b: i16 = get_string(&radios, CURRENT_BCR1_B)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;

        let voltage_bcr2: i16 = (get_string(&radios, VOLTAGE_BCR2)
            .parse::<f64>()
            .unwrap_or(0.0)
            * 1000.0) as i16;
        let current_bcr2a: i16 = get_string(&radios, CURRENT_BCR2_A)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;
        let current_bcr2b: i16 = get_string(&radios, CURRENT_BCR2_B)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;

        let mut msg = vec![];
        let _ = msg.write_u16::<LittleEndian>(voltage_cell1);
        let _ = msg.write_u16::<LittleEndian>(voltage_cell2);
        let _ = msg.write_u16::<LittleEndian>(voltage_cell3);
        let _ = msg.write_u16::<LittleEndian>(voltage_cell4);
        let _ = msg.write_i16::<LittleEndian>(voltage_bcr1);
        let _ = msg.write_i16::<LittleEndian>(current_bcr1a);
        let _ = msg.write_i16::<LittleEndian>(current_bcr1b);
        let _ = msg.write_i16::<LittleEndian>(voltage_bcr2);
        let _ = msg.write_i16::<LittleEndian>(current_bcr2a);
        let _ = msg.write_i16::<LittleEndian>(current_bcr2b);

        let _ = radios.transmit(MessageType::Power, 2, &msg);

        let voltage_bcr6: i16 = (get_string(&radios, VOLTAGE_BCR6)
            .parse::<f64>()
            .unwrap_or(0.0)
            * 1000.0) as i16;
        let current_bcr6a: i16 = get_string(&radios, CURRENT_BCR6_A)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;
        let current_bcr6b: i16 = get_string(&radios, CURRENT_BCR6_B)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;

        let voltage_bcr7: i16 = (get_string(&radios, VOLTAGE_BCR7)
            .parse::<f64>()
            .unwrap_or(0.0)
            * 1000.0) as i16;
        let current_bcr7a: i16 = get_string(&radios, CURRENT_BCR7_A)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;
        let current_bcr7b: i16 = get_string(&radios, CURRENT_BCR7_B)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;

        let voltage_bcr8: i16 = (get_string(&radios, VOLTAGE_BCR8)
            .parse::<f64>()
            .unwrap_or(0.0)
            * 1000.0) as i16;
        let current_bcr8a: i16 = get_string(&radios, CURRENT_BCR8_A)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;
        let current_bcr8b: i16 = get_string(&radios, CURRENT_BCR8_B)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;

        let voltage_bcr9: i16 = (get_string(&radios, VOLTAGE_BCR9)
            .parse::<f64>()
            .unwrap_or(0.0)
            * 1000.0) as i16;
        let current_bcr9a: i16 = get_string(&radios, CURRENT_BCR9_A)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;
        let current_bcr9b: i16 = get_string(&radios, CURRENT_BCR9_B)
            .parse::<f64>()
            .unwrap_or(0.0) as i16;

        let mut msg = vec![];
        let _ = msg.write_i16::<LittleEndian>(voltage_bcr6);
        let _ = msg.write_i16::<LittleEndian>(current_bcr6a);
        let _ = msg.write_i16::<LittleEndian>(current_bcr6b);
        let _ = msg.write_i16::<LittleEndian>(voltage_bcr7);
        let _ = msg.write_i16::<LittleEndian>(current_bcr7a);
        let _ = msg.write_i16::<LittleEndian>(current_bcr7b);
        let _ = msg.write_i16::<LittleEndian>(voltage_bcr8);
        let _ = msg.write_i16::<LittleEndian>(current_bcr8a);
        let _ = msg.write_i16::<LittleEndian>(current_bcr8b);
        let _ = msg.write_i16::<LittleEndian>(voltage_bcr9);
        let _ = msg.write_i16::<LittleEndian>(current_bcr9a);
        let _ = msg.write_i16::<LittleEndian>(current_bcr9b);

        let _ = radios.transmit(MessageType::Power, 3, &msg);

        // Run every 15 minutes
        thread::sleep(Duration::from_secs(15 * 60));
    }
}
