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

// Gather telemetry from the Maryland Aerospace MAI-400

use crate::telem_db::{process_json, send_telem};
use failure::{bail, Error};
use kubos_app::*;
use std::time::Duration;

const MAI_NOMINAL: &str = r#"{
    telemetry {
        nominal {
            gpsTime,
            timeSubsec,
            cmdValidCntr,
            cmdInvalidCntr,
            cmdInvalidChksumCntr,
            lastCommand,
            acsMode,
            css,
            eclipseFlag,
            sunVecB,
            iBFieldMeas,
            bd,
            rwsSpeedCmd,
            rwsSpeedTach,
            rwaTorqueCmd,
            gcRwaTorqueCmd,
            torqueCoilCmd,
            gcTorqueCoilCmd,
            qboCmd,
            qboHat,
            angleToGo,
            qError,
            omegaB,
            nb,
            neci,
        }
    }
}"#;
const MAI_DEBUG: &str = r#"{
    telemetry {
        debug {
            irehs {
                thermopilesA,
                thermopilesB,
                tempA,
                tempB,
                dipAngleA,
                dipAngleB,
                solutionDegraded,
                thermopileStructA {
                    dipAngle,
                    earthLimb {
                       adc,
                       temp,
                       errors,
                       flags,
                    },
                    earthRef {
                       adc,
                       temp,
                       errors,
                       flags,
                    },
                    spaceRef {
                       adc,
                       temp,
                       errors,
                       flags,
                    },
                    wideFov {
                       adc,
                       temp,
                       errors,
                       flags,
                    },
                },
                thermopileStructB {
                    dipAngle,
                    earthLimb {
                       adc,
                       temp,
                       errors,
                       flags,
                    },
                    earthRef {
                       adc,
                       temp,
                       errors,
                       flags,
                    },
                    spaceRef {
                       adc,
                       temp,
                       errors,
                       flags,
                    },
                    wideFov {
                       adc,
                       temp,
                       errors,
                       flags,
                    },
               },
            },
            rawImu {
                accel,
                gyro,
                gyroTemp,
            }
        }
    }
}"#;
const MAI_ROTATING: &str = r#"{
    telemetry {
        debug{
            rotating {
                bFieldIgrf,
                sunVecEph,
                scPosEci,
                scVelEci,
                keplerElem {
                    semiMajorAxis,
                    eccentricity,
                    inclination,
                    raan,
                    argParigee,
                    trueAnomoly,
                },
                kBdot,
                kp,
                kd,
                kUnload,
                cssBias,
                magBias,
                rwsVolt,
                rwsPress,
                attDetMode,
                rwsResetCntr,
                sunMagAligned,
                minorVersion,
                maiSn,
                orbitPropMode,
                acsOpMode,
                procResetCntr,
                majorVersion,
                adsOpMode,
                cssGain,
                magGain,
                orbitEpoch,
                trueAnomolyEpoch,
                orbitEpochNext,
                scPosEciEpoch,
                scVelEciEpoch,
                qbXWheelSpeed,
                qbXFilterGain,
                qbXDipoleGain,
                dipoleGain,
                wheelSpeedBias,
                cosSunMagAlignThresh,
                unloadAngThresh,
                qSat,
                rwaTrqMax,
                rwsMotorCurrent,
                rwsMotorTemp,
            }
        }
    }
}"#;

const MAI_POWER: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "AIM:ADCS:POW ON") {
            status,
            command
        }
    }
"#;

const MAI_COMM: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "AIM:ADCS:COMM UART0") {
            status,
            command
        }
    }
"#;

const MAI_PASS: &str = r#"
    mutation {
        passthrough(module: "aim2", command: "AIM:ADCS:PASS ON") {
            status,
            command
        }
    }
"#;

pub fn get_telem() -> Result<(), Error> {
    // Make sure the MAI-400 is on and able to communicate with us
    let service = ServiceConfig::new("pumpkin-mcu-service");

    let _ = query(&service, MAI_POWER, Some(Duration::from_millis(500)))?;

    let _ = query(&service, MAI_COMM, Some(Duration::from_millis(500)))?;

    let _ = query(&service, MAI_PASS, Some(Duration::from_millis(500)))?;

    let service = ServiceConfig::new("mai400-service");

    // The MAI-400 has a *bunch* of fields, so we're going to break this into three chunks to
    // help reduce the amount of data returned at one time

    // Get the nominal telemetry

    let result = query(&service, MAI_NOMINAL, Some(Duration::from_secs(2)))?;

    if result["telemetry"]["nominal"]["gpsTime"] == 0 {
        bail!("MAI-400 offline");
    }

    let nominal = &result["telemetry"]["nominal"].as_object();

    let mut telem_vec: Vec<(String, String)> = vec![];

    // Auto-convert returned JSON into a flat key-value vector
    if let Some(data) = nominal {
        process_json(&mut telem_vec, data, "".to_owned());
        // Send it to the telemetry database
        send_telem("MAI400", telem_vec);
    }

    // Get the debug telemetry, minus the rotating variables
    let result = query(&service, MAI_DEBUG, Some(Duration::from_secs(2)))?;

    let debug = &result["telemetry"]["debug"].as_object();

    let mut telem_vec: Vec<(String, String)> = vec![];

    // Auto-convert returned JSON into a flat key-value vector
    if let Some(data) = debug {
        process_json(&mut telem_vec, data, "".to_owned());
        // Send it to the telemetry database
        send_telem("MAI400", telem_vec);
    }

    // Get the rotating variables telemetry
    let result = query(&service, MAI_ROTATING, Some(Duration::from_secs(2)))?;

    let rotating = &result["telemetry"]["debug"]["rotating"].as_object();

    let mut telem_vec: Vec<(String, String)> = vec![];

    // Auto-convert returned JSON into a flat key-value vector
    if let Some(data) = rotating {
        process_json(&mut telem_vec, data, "".to_owned());
        // Send it to the telemetry database
        send_telem("MAI400", telem_vec);
    }

    Ok(())
}
