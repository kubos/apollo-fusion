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

// Helper functions to breakup and store data returned from the subsystems

use kubos_app::*;
use serde_json::{json, ser};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

// Send a list of key/value pairs to the telemetry database via the direct UDP port
pub fn send_telem(subsystem: &str, telem_vec: Vec<(String, String)>) {
    let config = ServiceConfig::new("telemetry-service");

    let port = config
        .get("direct_port")
        .expect("No `direct_port` param given");;

    let host = config.hosturl().to_owned();
    let ip: Vec<&str> = host.split(':').collect();

    let remote_addr = format!("{}:{}", ip[0], port);

    let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

    let socket = UdpSocket::bind(local_addr).expect("Couldn't bind to address");

    for (key, value) in telem_vec.iter() {
        let message = json!({
            "subsystem": subsystem,
            "parameter": key,
            "value": value
        });

        socket
            .send_to(&ser::to_vec(&message).unwrap(), &remote_addr)
            .unwrap();

        // Give the telemetry service just a little bit of breathing room
        thread::sleep(Duration::from_millis(1));
    }
}

// Convert an unknown list of telemetry in JSON format into a flat-structured set of
// key/value pairs.
//
// Example:
// Input -
// {
//     telemetry {
//         power {
//            voltage: 5,
//            current: 0.3
//        },
//        status: "Okay",
//        position: [1.3, -4.5, 9.0]
//    }
// }
//
// Output -
// [
//    ("telemetry_power_voltage", "5"),
//    ("telemetry_power_current", "0.3"),
//    ("telemetry_status", "Okay"),
//    ("telemetry_position_0", "1.3"),
//    ("telemetry_position_1", "-4.5"),
//    ("telemetry_position_2", "9.0")
// ]
pub fn process_json(
    mut telem_vec: &mut Vec<(String, String)>,
    data: &serde_json::Map<String, serde_json::Value>,
    prefix: String,
) {
    for (key, value) in data.iter() {
        match value {
            serde_json::Value::Object(object) => {
                process_json(&mut telem_vec, object, format!("{}{}_", prefix, key))
            }
            serde_json::Value::Array(array) => {
                for (index, val) in array.iter().enumerate() {
                    telem_vec.push((format!("{}{}_{}", prefix, key, index), format!("{}", val)))
                }
            }
            _ => telem_vec.push((format!("{}{}", prefix, key), format!("{}", value))),
        }
    }
}
