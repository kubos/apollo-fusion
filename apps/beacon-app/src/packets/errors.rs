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

// Pull the current number of errors and the last error from both the applications and services
// log files every 15 minutes
//
// Notes:
//   - All multi-byte fields are Little Endian
//   - If there are no error messages in a log file, only the errors count field will be returned
//     for that message
//
// Packet 1 (Application Errors. 34 bytes):
// 0-1: Errors count
// 2-3: UTC timestamp from last error, in seconds, reduced from u64 to u16 for compactness
// 4-11: Source of error message, truncated to 8 bytes
// 12-33: Error message, truncated to 22 bytes
//
// Packet 2 (Service Errors. 34 bytes):
// - Same as packet 1

use crate::transmit::*;
use byteorder::{LittleEndian, WriteBytesExt};
use chrono::prelude::*;
use std::process::Command;
use std::thread;
use std::time::Duration;

const APP_ERRORS_FILE: &str = "/var/log/app-warn.log";
const SERVICE_ERRORS_FILE: &str = "/var/log/kubos-warn.log";

pub fn errors_packet(radios: Radios) {
    loop {
        let app_msg = create_errors_message(APP_ERRORS_FILE);

        let _ = radios.transmit(MessageType::Errors, 1, &app_msg);

        let app_msg = create_errors_message(SERVICE_ERRORS_FILE);

        let _ = radios.transmit(MessageType::Errors, 2, &app_msg);

        thread::sleep(Duration::from_secs(15 * 60));
    }
}

fn create_errors_message(file: &str) -> Vec<u8> {
    // Get the total number of lines in the error file to estimate the current errors count
    let output = Command::new("wc")
        .args(&["-l", file])
        .output()
        .map(|output| output.stdout)
        .unwrap_or_else(|_| vec![])
        .iter()
        .filter_map(|&elem| {
            if elem.is_ascii_digit() {
                Some(elem as char)
            } else {
                None
            }
        })
        .collect::<String>();

    let errors_count: u16 = output.parse().unwrap_or(0xFFFF);

    // Get the last error message from the file
    let output = Command::new("tail")
        .args(&["-n", "1", file])
        .output()
        .map(|output| output.stdout)
        .unwrap_or_else(|_| vec![]);
    let last_error = std::str::from_utf8(&output)
        .unwrap_or_else(|_| "")
        .to_owned();

    let mut pieces = last_error.split('>');
    let mut header = pieces.next().unwrap_or("").split(' ');

    // Get the UTC time (time since Unix Epoch in seconds)
    // Shrink to a u16 to fit in the message
    let timestamp: u16 = header
        .next()
        .and_then(|raw| {
            raw.parse::<DateTime<Utc>>()
                .map(|value| value.timestamp())
                .ok()
        })
        .unwrap_or(0) as u16;

    let _kubos = header.next();

    // Get the message source. Shorten to 8 bytes
    let source = header
        .next()
        .map(|text| if text.len() > 7 { &text[0..8] } else { text })
        .unwrap_or("");

    // Get the actual error message. Skip the first byte (it's a space). Truncate to 22 bytes
    let text = pieces
        .next()
        .map(|text| if text.len() > 22 { &text[1..23] } else { text })
        .unwrap_or("");

    let mut msg = vec![];
    let _ = msg.write_u16::<LittleEndian>(errors_count);
    let _ = msg.write_u16::<LittleEndian>(timestamp);
    msg.extend_from_slice(source.as_bytes());
    msg.extend_from_slice(text.as_bytes());

    msg
}
