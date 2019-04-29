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

// GraphQL requests

use std::time::Duration;

pub const QUERY_TIMEOUT: Duration = Duration::from_millis(500);

// Turn on the MAI-400
pub const MAI_POWER: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "AIM:ADCS:POW ON") {
            status
        }
    }
"#;

// Configure the MAI-400's UART
// BBB UART5 = CSK UART0
pub const MAI_COMM: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "AIM:ADCS:COMM UART0") {
            status
        }
    }
"#;

// Enable communication with the MAI-400
pub const MAI_PASS: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "AIM:ADCS:PASS ON") {
            status
        }
    }
"#;

// Turn on the OEM7
pub const OEM_POWER: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "GPS:POW ON") {
            status
        }
    }
"#;

// Configure the OEM's UART
// BBB UART4 = CSK UART3
pub const OEM_COMM: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "GPS:COMM UART3") {
            status
        }
    }
"#;

// Enable communication with the OEM
pub const OEM_PASS: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "GPS:PASS ON") {
            status
        }
    }
"#;

// TODO: How frequently do we want the OEM to send us position data?
// Set up the OEM logs that we care about
pub const OEM_SET_LOGS: &str = r#"
    mutation {
        configureHardware(config: [
            {option: UNLOG_ALL, hold: true},
            {option: LOG_ERROR_DATA},
            {option: LOG_POSITION_DATA, interval: 60.0}
        ]) {
            success,
            errors
        }
    }
"#;

// Turn on the duplex radio
pub const DUPLEX_POWER: &str = r#"
    mutation {
        passthrough(module: "bim", command: "BIM:UART:POW 2,ON") {
            status
        }
    }
"#;

// Turn on the simplex radio
pub const SIMPLEX_POWER: &str = r#"
    mutation {
        passthrough(module: "rhm", command: "RHM:GS:POW ON") {
            status
        }
    }
"#;

// Configure the simplex's UART
// BBB UART3 = CSK UART4
pub const SIMPLEX_COMM: &str = r#"
    mutation {
        passthrough(module: "rhm", command: "RHM:GS:COMM UART4") {
            status
        }
    }
"#;

// Enable simplex communication
pub const SIMPLEX_PASS: &str = r#"
    mutation {
        passthrough(module: "rhm", command: "RHM:GS:PASS ON") {
            status
        }
    }
"#;

// Kick off the H&S beacon application
pub const START_BEACON: &str = r#"
    mutation {
        startApp(name: "beacon-app", runLevel: "OnCommand") {
            success,
            errors
        }
    }
"#;

// Kick off the ADCS housekeeping application
pub const START_ADCS: &str = r#"
    mutation {
        startApp(name: "adcs-app", runLevel: "OnCommand") {
            success,
            errors
        }
    }
"#;

// Enable the TiNi pin puller
pub const DEPLOY_ENABLE: &str = r#"
    mutation {
        passthrough(module: "bim", command: "BIM:TINI ENAB") {
            status
        }
    }
"#;

// Arm the pin puller
pub const DEPLOY_ARM: &str = r#"
    mutation {
        passthrough(module: "bim", command: "BIM:TINI ARM") {
            status
        }
    }
"#;

// Energize the pin puller, which should cause the solar panels to be released
pub const DEPLOY_FIRE: &str = r#"
    mutation {
        passthrough(module: "bim", command: "BIM:TINI FIRE,30") {
            status
        }
    }
"#;
