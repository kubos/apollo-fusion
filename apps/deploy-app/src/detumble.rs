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

// TODO: Most of the actual logic isn't in place yet

use crate::graphql::*;
use kubos_app::*;
use log::*;
use std::thread;
use std::time::Duration;

pub fn detumble_wait() {
    // Wait until the spin rates are in an acceptable range.
    // Max wait time: 12 hours
    // TODO: Update to check for time passed because of reboots
    for _ in 0..144 {
        // Check ADCS spin rates
        // If acceptable, break out of this loop

        // Otherwise, wait for 5 minutes before trying again
        thread::sleep(Duration::from_secs(300));
    }

    let mai_service = ServiceConfig::new("mai400-service");
    // Update MAI-400 with latest GPS lock

    // Do prep for safe mode:
    // - Initialize GPS time + ephemeris

    // TODO: Get GPS time
    // OEM6. Get lockInfo{time{ms,week}} and lockStatus{timeStatus} to verify validity

    /*
    let mutation = format!(r#"
        mutation {{
            update(gps_time: {}, rv: {{eciPos: {}, eciVel: {}, timeEpoch: {}}}) {{
                success,
                errors
            }}
        }}
    "#, gps_time, eci_pos, eci_vel, time_epoch);

    if let Err(error) = query(&mai_service, mutation, Some(QUERY_TIMEOUT)) {
        error!("Failed to set MAI-400's GPS time and/or ephemeris: {:?}", error);
        // TODO: Is there a reason to keep going if this fails?
        return;
    }

    debug!("New MAI-400 GPS time: {}", gps_time);
    */
    // - Initialize the ephemeris

    // Go into "safe" mode (normal mode?)
    if let Err(error) = query(&mai_service, MAI_NORMAL_MODE, Some(QUERY_TIMEOUT)) {
        error!("Failed to put MAI-400 in normal mode: {:?}", error);
    }
}
