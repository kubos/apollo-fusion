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

pub const MAI_POWER: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "AIM:ADCS:POW ON") {
            status,
            command
        }
    }
"#;

pub const MAI_COMM: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "AIM:ADCS:COMM UART0") {
            status,
            command
        }
    }
"#;

pub const MAI_PASS: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "AIM:ADCS:PASS ON") {
            status,
            command
        }
    }
"#;

pub const OEM6_POWER: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "GPS:POW ON") {
            status,
            command
        }
    }
"#;

pub const OEM6_COMM: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "GPS:COMM UART3") {
            status,
            command
        }
    }
"#;

pub const OEM6_PASS: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "GPS:PASS ON") {
            status,
            command
        }
    }
"#;

pub const SIMPLEX_POWER: &str = r#"
    mutation {
        passthrough(module: "rhm", command: "RHM:POW ON") {
            status,
            command
        }
    }
"#;

// TODO: Verify UART
pub const SIMPLEX_COMM: &str = r#"
    mutation {
        passthrough(module: "rhm", command: "RHM:COMM UART1") {
            status,
            command
        }
    }
"#;

pub const SIMPLEX_PASS: &str = r#"
    mutation {
        passthrough(module: "rhm", command: "RHM:PASS ON") {
            status,
            command
        }
    }
"#;

pub const START_BEACON: &str = r#"
    mutation {
        startApp(name: "beacon-app", runLevel: "OnCommand") {
            success,
            errors
        }
    }
"#;