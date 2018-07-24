// Copyright (C) 2018 Kubos Corporation. All rights reserved.
#[macro_use]
extern crate kubos_app;
extern crate kubos_system;
extern crate kubos_telemetry_db;
#[macro_use]
extern crate lazy_static;
extern crate serde_json;

use std::thread::sleep;
use std::time::Duration;

use kubos_system::Config;

mod services;

struct ApolloFusionApp {
    telemetry: kubos_telemetry_db::Database,
}

impl kubos_app::AppHandler for ApolloFusionApp {
    fn on_boot(&self) {
        println!("ON-BOOT");
        self.print_boot_vars();
        if let Some(initial_deploy) = kubos_system::initial_deploy() {
            if initial_deploy {
                self.on_initial_deploy();
            }
        }

        self.main_loop();
    }

    fn on_command(&self) {
        println!("ON-COMMAND");
        self.print_boot_vars();
    }
}

impl ApolloFusionApp {
    pub fn new() -> ApolloFusionApp {
        let config = Config::new("telemetry-service");
        let db_path = config
            .get("database")
            .expect("No database path found in config file");
        let db_path = db_path.as_str().unwrap_or("");

        let telemetry = kubos_telemetry_db::Database::new(&db_path);
        telemetry.setup();

        ApolloFusionApp {
            telemetry: telemetry,
        }
    }

    pub fn print_boot_vars(&self) {
        let versions = kubos_system::kubos_versions();
        if let Some(curr) = versions.curr {
            println!("KubOS Curr Version: {}", curr);
        }

        if let Some(prev) = versions.prev {
            println!("KubOS Prev Version: {}", prev);
        }

        if let Some(initial_deploy) = kubos_system::initial_deploy() {
            println!("KubOS Initial Deploy: {}", initial_deploy);
        }
    }

    pub fn main_loop(&self) {
        // Turn on GPSRM and enable pass-through
        self.gps_passthrough("GPS:POW ON").expect("Failed to power on GPS");
        self.gps_passthrough("GPS:PASS ON").expect("Failed to enable passthrough on GPS");

        loop {
            // Record the current GPS subsystem power state
            let gps_state = self.gps_power_state();
            if let Err(err) = self.telemetry.insert_systime(
                "gps",
                "power.state",
                if gps_state { "ON" } else { "OFF" },
            ) {
                eprintln!("Warning: failed to insert GPS state: {:?}", err);
            }

            sleep(Duration::from_secs(60));
        }
    }

    pub fn on_initial_deploy(&self) {}

    pub fn gps_passthrough(&self, command: &str) -> Result<serde_json::Value, String> {
        match services::query(
            services::MCU_SERVICE,
            &format!(
                r#"mutation {{
                         passthrough(module: "gpsrm",
                                     command: "{}") {{ status, command }} }}"#,
                command
            ),
            2,
        ) {
            Ok(value) => {
                let passthrough = value.get("passthrough");
                match passthrough {
                    Some(p) => Ok(p.clone()),
                    None => Err(String::from("No passthrough result")),
                }
            }
            Err(e) => Err(format!("{}", e)),
        }
    }

    pub fn gps_power_state(&self) -> bool {
        match services::query(services::GPS_SERVICE, "{ power { state } }", 2) {
            Ok(value) => match value.get("power") {
                Some(power) => {
                    let state = power["state"].as_str().unwrap_or("OFF");
                    match state.to_lowercase().as_ref() {
                        "on" => true,
                        _ => false,
                    }
                }
                _ => false,
            },
            Err(e) => {
                println!("Error querying GPS: {:?}", e);
                false
            }
        }
    }
}

fn main() {
    app_main!(&ApolloFusionApp::new());
}
