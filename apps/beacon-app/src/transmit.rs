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

// Module for actually sending messages
//
// All messages are sent over both the simplex and the duplex

use failure::{bail, Error};
use kubos_app::{query, ServiceConfig};
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct Radios {
    pub telem_service: ServiceConfig,
    pub simplex: Arc<Mutex<ServiceConfig>>,
    // TODO: duplex: DuplexD2,
}

pub const SIMPLEX_STATUS: &str = r#"{
    mcuTelemetry(module: "rhm", fields: ["globalstar_status"])
}"#;

impl Radios {
    pub fn transmit(&self, msg_type: MessageType, subtype: u8, data: &[u8]) -> Result<(), Error> {
        // Combine message type and subtype into single header byte
        // 7 6 5 4 3 | 2 1 0
        //  Msg type | Sub type
        let header: u8 = ((msg_type as u8) << 3) | subtype;

        if data.len() > 34 {
            bail!("Message too long");
        }

        // Create full message packet
        let mut packet = vec![header];
        packet.extend_from_slice(data);

        // Send the packet
        if let Err(error) = self.send_simplex(&packet) {
            error!("Failed to send beacon over simplex: {:?}", error);
        }
        if let Err(error) = self.send_duplex(&packet) {
            error!("Failed to send beacon over duplex: {:?}", error);
        }
        Ok(())
    }

    // Note: This send logic is configured to be sent via the RHM supMCU module.
    // It's possible for the simplex to be connected directly to a UART.
    // If that happens (ie if we want to re-use this code in the future), this logic will need to be
    // updated to use a standard UART connection for communication.
    fn send_simplex(&self, packet: &[u8]) -> Result<(), Error> {
        // We've got the service config in a mutex to ensure messages get sent one at a time
        // If the mutex gets poisoned, we want to crash as noisily as possible
        let simplex = self.simplex.lock().unwrap();

        // If the simplex is currently sending a message, it can take up to 10 seconds for it to
        // finish and be ready for new data
        for _ in 0..15 {
            // Get the current status of the simplex
            let status = self.check_simplex(&simplex)?;

            if status == SimplexStatus::NoTransmissions
                || status == SimplexStatus::LastGood
                || status == SimplexStatus::LastBad
            {
                break;
            }

            thread::sleep(Duration::from_secs(1));
        }

        debug!("Sending packet over simplex: {:#02x?}", packet);

        let hex: String = packet
            .iter()
            .map(|elem| format!("{:02x}", elem))
            .collect::<Vec<String>>()
            .join("");

        let request = format!(
            r#"mutation {{
                passthrough(module: "rhm", command: "RMS:GS:SEND {}") {{
                    status,
                    command
                }}
            }}"#,
            hex
        );

        let result = query(&simplex, &request, Some(Duration::from_millis(200)))?;
        let status = result["passthrough"]["status"].as_bool().unwrap_or(false);
        if !status {
            bail!("Failed to send packet to RHM");
        }

        // It'll take roughly 10 seconds for the message to be sent and then the simplex's Busy
        // line to go low
        thread::sleep(Duration::from_secs(10));

        let status = self.check_simplex(&simplex)?;
        match status {
            SimplexStatus::LastGood => Ok(()),
            other => {
                bail!("Bad simplex response: {:?}", other);
            }
        }
    }

    // Ask the RHM supMCU for the current status of the simplex
    fn check_simplex(&self, simplex: &ServiceConfig) -> Result<SimplexStatus, Error> {
        let result = query(simplex, SIMPLEX_STATUS, Some(Duration::from_millis(300)))?;

        let telem_raw = result["mcuTelemetry"].as_str().unwrap_or("");
        let telem: serde_json::Value = serde_json::from_str(telem_raw)?;
        let status = if let Some(value) = telem
            .as_object()
            .and_then(|obj| obj.get("globalstar_status"))
        {
            if value["timestamp"] != 0 {
                match value["data"].as_u64().unwrap_or(0xFF) {
                    0 => SimplexStatus::NoTransmissions,
                    1 => SimplexStatus::Busy,
                    2 => SimplexStatus::Prepping,
                    3 => SimplexStatus::Transmitting,
                    4 => SimplexStatus::LastGood,
                    5 => SimplexStatus::LastBad,
                    _ => SimplexStatus::Unknown,
                }
            } else {
                SimplexStatus::Unknown
            }
        } else {
            SimplexStatus::Unknown
        };

        Ok(status)
    }

    fn send_duplex(&self, packet: &[u8]) -> Result<(), Error> {
        debug!("Sending packet over duplex: {:?}", packet);
        Ok(())
    }
}

pub enum MessageType {
    ADCS = 0,
    Errors = 1,
    GPS = 2,
    OBC = 3,
    Power = 4,
    Radio = 5,
    SupMCU = 6,
    Temperature = 7,
}

// Current status of the simplex, as reported by the RHM supMCU
#[derive(Debug, PartialEq)]
enum SimplexStatus {
    // Nothing has been sent (since last reset?)
    NoTransmissions,
    // Simplex's busy line is high
    Busy,
    // Simplex is preparing to transmit a message
    Prepping,
    // Simplex is actively transmitting a message
    Transmitting,
    // The last transmission was successful
    LastGood,
    // The last transmission failed
    LastBad,
    // An unknown status value was received
    Unknown,
}
