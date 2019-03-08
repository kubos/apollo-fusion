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

use failure::Error;
use kubos_app::*;
use kubos_system::Config;
use serde_json::{json, ser};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

const MAI_TELEMETRY: &str = r#"{
    telemetry{
        nominal{
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
        },
        debug{
            irhes{
                thermopilesA,
                thermopilesB,
                tempA,
                tempB,
                dipAngleA,
                dipAngleB,
                solutionDegraded,
                thermopileStructA{
                    dipAngle,
                    earthLimb: {
                       adc,
   		               temp,
   		               errors,
   		               flags,
    	               },
                    earthRef: {
						adc,
   		               temp,
   		               errors,
   		               flags,
    	               },
                    spaceRef: {
						adc,
   		               temp,
   		               errors,
   		               flags,
    	               },
                    wideFov: {
						adc,
   		               temp,
   		               errors,
   		               flags,
    	               },
                thermopileStructB{
						adc,
   		               temp,
   		               errors,
   		               flags,
    	               },
			   }
            },
            rawImu{
                accel,
                gyro,
                gyroTemp,
            },
            rotating{
                bFieldIgrf,
                sunVecEph,
                scPosEci,
                scVelEci,
                keplerElem{
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

pub fn get_telem() -> Result<(), Error> {
    let service = ServiceConfig::new("mai400-service");
    
    // Get all the basic telemetry
    /*
    let result = query(
        &service,
        MAI_TELEMETRY,
        Some(Duration::from_millis(100)
    ))?;
    */
    
    let result = json!({ "data": {
    	"telemetry": {
    		"nominal": {
                "gpsTime": 0,
                "timeSubsec": 0,
                "cmdValidCntr": 0,
                "cmdInvalidCntr": 0,
                "cmdInvalidChksumCntr": 0,
                "lastCommand": 0,
                "acsMode": 0,
                "css": 0,
                "eclipseFlag": 0,
                "sunVecB": 0,
                "iBFieldMeas": 0,
                "bd": 0,
                "rwsSpeedCmd": 0,
                "rwsSpeedTach": 0,
                "rwaTorqueCmd": 0,
                "gcRwaTorqueCmd": 0,
                "torqueCoilCmd": 0,
                "gcTorqueCoilCmd": 0,
                "qboCmd": 0,
                "qboHat": 0,
                "angleToGo": 0,
                "qError": 0,
                "omegaB": 0,
                "nb": 0,
                "neci": 0,
            },
	        "debug": {
                "irhes": {
                    "thermopilesA": 1,
                    "thermopilesB": 1,
                    "tempA": 1,
                    "tempB": 1,
                    "dipAngleA": 1,
                    "dipAngleB": 1,
                    "solutionDegraded": 1,
                    "thermopileStructA": {
                        "dipAngle": 1,
                        "earthLimb": {
                           "adc": 1,
                              "temp": 1,
                              "errors": 1,
                              "flags": 1,
                           },
                        "earthRef": {
                            "adc": 1,
                              "temp": 1,
                              "errors": 1,
                              "flags": 1,
                           },
                        "spaceRef": {
                            "adc": 1,
                              "temp": 1,
                              "errors": 1,
                              "flags": 1,
                           },
                        "wideFov": {
                            "adc": 1,
                              "temp": 1,
                              "errors": 1,
                              "flags": 1,
                           },
                    "thermopileStructB": {
                            "adc": 1,
                              "temp": 1,
                              "errors": 1,
                              "flags": 1,
                           },
                   }
                },
                "rawImu": {
                    "accel": 1,
                    "gyro": 1,
                    "gyroTemp": 1,
                },
                "rotating": {
                    "bFieldIgrf": 1,
                    "sunVecEph": 1,
                    "scPosEci": 1,
                    "scVelEci": 1,
                    "keplerElem": {
                        "semiMajorAxis": 1,
                        "eccentricity": 1,
                        "inclination": 1,
                        "raan": 1,
                        "argParigee": 1,
                        "trueAnomoly": 1,
                    },
                    "kBdot": 1,
                    "kp": 1,
                    "kd": 1,
                    "kUnload": 1,
                    "cssBias": 1,
                    "magBias": 1,
                    "rwsVolt": 1,
                    "rwsPress": 1,
                    "attDetMode": 1,
                    "rwsResetCntr": 1,
                    "sunMagAligned": 1,
                    "minorVersion": 1,
                    "maiSn": 1,
                    "orbitPropMode": 1,
                    "acsOpMode": 1,
                    "procResetCntr": 1,
                    "majorVersion": 1,
                    "adsOpMode": 1,
                    "cssGain": 1,
                    "magGain": 1,
                    "orbitEpoch": 1,
                    "trueAnomolyEpoch": 1,
                    "orbitEpochNext": 1,
                    "scPosEciEpoch": 1,
                    "scVelEciEpoch": 1,
                    "qbXWheelSpeed": 1,
                    "qbXFilterGain": 1,
                    "qbXDipoleGain": 1,
                    "dipoleGain": 1,
                    "wheelSpeedBias": 1,
                    "cosSunMagAlignThresh": 1,
                    "unloadAngThresh": 1,
                    "qSat": 1,
                    "rwaTrqMax": 1,
                    "rwsMotorCurrent": 1,
                    "rwsMotorTemp": [1, 2, 3, 4],
                }
            }
        }
    }});
    
    let mut telem_vec: Vec<(String, String)> = vec!();
    let nominal = &result["data"]["telemetry"]["nominal"].as_object();
    let debug = &result["data"]["telemetry"]["debug"].as_object();
    
    // Auto-convert returned JSON into a flat key-value vector
    if let Some(data) = nominal {
        process_json(&mut telem_vec, data, "".to_owned());
    }
    
    if let Some(data) = debug {
        process_json(&mut telem_vec, data, "".to_owned());
    }
    
    // Send it to the telemetry database
    send_telem("mai400", telem_vec);
    
    Ok(())
}

fn send_telem(subsystem: &str, telem_vec: Vec<(String, String)>) {
    let config = Config::new("telemetry-service");
    
    let port = config.get("direct_port").unwrap();
    
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
    }
}

fn process_json(mut telem_vec: &mut Vec<(String, String)>, data: &serde_json::Map<String, serde_json::Value>, prefix: String) {
    for (key, value) in data.iter() {
        match value {
            serde_json::Value::Object(object) => process_json(&mut telem_vec, object, format!("{}{}_", prefix, key)),
            serde_json::Value::Array(array) => {
                for (index, val) in array.iter().enumerate() {
                    telem_vec.push((format!("{}{}_{}", prefix, key, index), format!("{}", val)))
                }
            }
            _ => telem_vec.push((format!("{}{}", prefix, key), format!("{}", value)))
        }
    }
}