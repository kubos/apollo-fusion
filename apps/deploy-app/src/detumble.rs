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
use crate::deploy::*;
use crate::graphql::*;
use kubos_app::*;
use log::*;
use std::thread;
use std::time::Duration;

pub fn detumble_wait() {
    for _ in 0..144 {
        // Check ADCS spin rates
        // If acceptable, break out of this loop
        
        // Otherwise, wait for 5 minutes before trying again
        thread::sleep(Duration::from_secs(300));
    }
    
    // Do prep for safe mode:
    // - Initialize GPS time
    // - Initialize the ephemeris    
    
    // Go into "safe" mode (normal mode?)
}