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

// Check the current RAM and disk usage
//
// - If RAM usage is too high (> 80%), reboot the system
// - If disk usage is too high (> 80%), delete all telemetry database entries older than a day
//   (vs the usual week lifespan)

use super::*;
use std::process::Command;

const OBC_TELEMETRY: &str = r#"{
    memInfo {
        available
    }
}"#;
// Taken from /proc/meminfo on a BBB
const MEM_TOTAL: u64 = 515_340;

pub fn check_mem() -> Result<(), Error> {
    // Check RAM usage
    let service = ServiceConfig::new("monitor-service");

    let result = query(&service, OBC_TELEMETRY, Some(Duration::from_secs(1)))?;

    let mem = result["memInfo"]["available"].as_u64().unwrap_or(0);

    // Convert to percentage in use, since that's an easier number to work with
    let ram_in_use = 100 - mem * 100 / MEM_TOTAL;

    // TODO: Decide on thresholds
    if ram_in_use < 50 {
        info!("RAM usage nominal: {}%", ram_in_use);
    } else if ram_in_use < 70 {
        info!("RAM usage high, but acceptable: {}%", ram_in_use);
    } else if ram_in_use < 80 {
        warn!("RAM usage high: {}%", ram_in_use);
    } else {
        error!("RAM usage too high: {}%. Triggering reboot", ram_in_use);
        // Reboot the system not-nicely. If we're at this point, there's probably a rogue process
        // that's hogging all the system resources and not playing nicely with others.
        Command::new("reboot").arg("-f").status()?;
    }

    // Check disk space usage
    // Get the % of the user data partition that's free
    //
    // Since we're using the MBM2, we'll need to check both of the possible disk names.
    // If the SD card is present, then the eMMC will be mmc1. Otherwise, it will be mmc0.
    //
    // Note: I tried to just use a wildcard ("/dev/mmcblk*p4"), but couldn't get the correct
    // output for some reason, so we're doing this the long way.
    let disk_in_use = if let Ok(output1) = Command::new("df").arg("/dev/mmcblk1p4").output() {
        let stdout = if output1.stderr.is_empty() {
            output1.stdout
        } else if let Ok(output0) = Command::new("df").arg("/dev/mmcblk0p4").output() {
            if output0.stderr.is_empty() {
                output0.stdout
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let mut slices = stdout.rsplit(|&elem| elem == b' ');

        // The last entry is the mount point (/home)
        slices.next();
        // The second to last entry is the percent in use
        let temp = slices.next();
        // Convert it to a useable number
        let percent = temp
            .unwrap_or(&[])
            .iter()
            .filter_map(|&elem| {
                if elem.is_ascii_digit() {
                    Some(elem as char)
                } else {
                    None
                }
            })
            .collect::<String>();

        percent.parse::<u8>().unwrap_or(100)
    } else {
        error!("Failed to get current disk usage info");
        100
    };

    if disk_in_use < 50 {
        info!("Disk usage nominal: {}%", disk_in_use);
    } else if disk_in_use < 70 {
        info!("Disk usage high, but acceptable: {}%", disk_in_use);
    } else if disk_in_use < 80 {
        warn!("Disk usage high: {}%", disk_in_use);
    } else {
        error!("Disk usage too high: {}%. Triggering cleanup", disk_in_use);
        // Delete everything from the database that's more than a day old (60*60*24)
        clean_db::clean_db(86400.0)?;
    }

    Ok(())
}
