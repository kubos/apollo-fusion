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

// Gather SupMCU module uptimes and reset flags every hour
//
// Message layout (18 bytes. All multi-byte fields are Little Endian):
//     0: AIM2 uptime
//   1-2: AIM2 reset flags
//     3: BIM uptime
//   4-5: BIM reset flags
//     6: PIM uptime
//   7-8: PIM reset flags
//     9: SIM uptime
// 10-11: SIM reset flags
//    12: RHM uptime
// 13-14: RHM reset flags
//    15: BM2 uptime
// 16-17: BM2 reset flags

// Reset Flags (Documentation provided by Pumpkin):
//     - 0 = Power on Reset (The board was just applied power or cycled power).
//     - 1 = Brown-out Reset (Unstable VCC into MCU caused reset)
//     - 2 = MCU woke from idle instr. (not useful)
//     - 3 = MCU woke from sleep instr. (not useful)
//     - 4 = WDTO Reset (Software Watchdog on MCU expired) [should not happen]
//     - 5 = SWDTEN (1 = software WDT is enabled, 0 = not enabled) [should always be 1]
//     - 6 = SWR Reset (SupMCU was instructed to reset) [SUP:RES NOW was sent as command]
//     - 7 = External Reset (Bus WDT reset the MCU module)
//     - 8 = VREGS (not used)
//     - 9 = Firmware configuration mismatch Reset (not used)
//     - 10 = not used (reads as 0)
//     - 11 = VREGSF (not used)
//     - 12-13 = not used (reads as 0)
//     - 14 = IOPUWR Reset (Illegal opcode executed) [should not happen]
//     - 15 = TRAPR (Trap Conflict/interrupt conflict) [should not happen]

use crate::transmit::*;
use byteorder::{LittleEndian, WriteBytesExt};
use kubos_app::query;
use std::thread;
use std::time::Duration;

pub fn supmcu_packet(radios: Radios) {
    let modules = ["aim2", "bim", "pim", "sim", "rhm", "bm2"];
    loop {
        let mut msg = vec![];
        for module in modules.iter() {
            let request = format!(
                r#"{{
                telemetry(subsystem: "{}", parameter: "time", limit: 1) {{
                    value
                }}
            }}"#,
                module
            );

            let uptime: u8 = if let Ok(data) = query(
                &radios.telem_service,
                &request,
                Some(Duration::from_millis(500)),
            ) {
                let raw = data["telemetry"][0]["value"].as_str().unwrap_or("");
                let conv = raw.parse::<u64>().unwrap_or(0);
                conv as u8
            } else {
                0
            };

            msg.push(uptime);

            let request = format!(
                r#"{{
                telemetry(subsystem: "{}", parameter: "reset_cause", limit: 1) {{
                    value
                }}
            }}"#,
                module
            );

            let reset: u16 = if let Ok(data) = query(
                &radios.telem_service,
                &request,
                Some(Duration::from_millis(500)),
            ) {
                let value = data["telemetry"][0]["value"]
                    .as_str()
                    .unwrap_or("")
                    .parse::<u16>()
                    .unwrap_or(0xFFFF);
                value as u16
            } else {
                0xFFFF
            };

            let _ = msg.write_u16::<LittleEndian>(reset);
        }

        let _ = radios.transmit(MessageType::SupMCU, 0, &msg);

        // Run every hour
        thread::sleep(Duration::from_secs(3600));
    }
}
