// Copyright (C) 2018 Kubos Corporation. All rights reserved.
#[macro_use]
extern crate kubos_app;
extern crate kubos_system;
extern crate kubos_telemetry;
#[macro_use]
extern crate lazy_static;
extern crate serde_json;

use std::thread::sleep;
use std::time::Duration;

use kubos_app::App;
use kubos_system::Config;

mod services;

struct ApolloFusionApp {
    telemetry: kubos_telemetry::Database,
}

impl kubos_app::AppHandler for ApolloFusionApp {
    fn on_boot(&self) {
        println!("ON-BOOT");
        self.print_boot_vars();
        if let Some(initial_deploy) = kubos_system::kubos_initial_deploy() {
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

        let telemetry = kubos_telemetry::Database::new(&db_path);
        telemetry.setup();

        ApolloFusionApp {
            telemetry: telemetry,
        }
    }

    pub fn print_boot_vars(&self) {
        if let Some(count) = kubos_system::boot_count() {
            println!("Boot Count: {}", count);
        }

        if let Some(limit) = kubos_system::boot_limit() {
            println!("Boot Limit: {}", limit);
        }

        if let Some(kubos_curr_version) = kubos_system::kubos_curr_version() {
            println!("KubOS Curr Version: {}", kubos_curr_version);
        }

        if let Some(kubos_prev_version) = kubos_system::kubos_prev_version() {
            println!("KubOS Prev Version: {}", kubos_prev_version);
        }

        if let Some(kubos_curr_tried) = kubos_system::kubos_curr_tried() {
            println!("KubOS Curr Tried: {}", kubos_curr_tried);
        }

        if let Some(initial_deploy) = kubos_system::kubos_initial_deploy() {
            println!("KubOS Initial Deploy: {}", initial_deploy);
        }
    }

    pub fn main_loop(&self) {
        // Turn on GPSRM and enable pass-through
        self.gps_passthrough("GPS:POW ON");
        self.gps_passthrough("GPS:PASS ON");

        loop {
            // Record the current GPS subsystem power state
            let gps_state = self.gps_power_state();
            self.telemetry.insert_systime(
                "gps",
                "power.state",
                if gps_state { "ON" } else { "OFF" },
            );

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
