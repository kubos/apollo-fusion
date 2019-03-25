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

use failure::{bail, Error};
use kubos_app::ServiceConfig;
use log::*;
use nsl_simplex_s3::SimplexS3;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Radios {
    pub telem_service: ServiceConfig,
    pub simplex: SimplexS3,
    // TODO: duplex: DuplexD2,
}

impl Radios {
    pub fn transmit(&self, msg_type: MessageType, subtype: u8, data: &[u8]) -> Result<(), Error> {
        // TODO: Convert message type enum to value
        // TODO: Combine message type and subtype into single header byte
        let header: u8 = msg_type as u8;

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

    fn send_simplex(&self, packet: &[u8]) -> Result<(), Error> {
        debug!("Sending packet over simplex: {:?}", packet);
        Ok(())
        //self.simplex.send_beacon(packet)
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
