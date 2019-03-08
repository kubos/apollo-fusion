#![recursion_limit="256"]
//mod eps;
mod mai400;
mod nsl;
mod oem6;
mod sup_mcu;
mod telem_db;

use failure::{bail, Error};
use getopts::Options;
use kubos_app::*;
use log::*;
use std::thread;
use std::time::Duration;

struct MyApp;

impl AppHandler for MyApp {
    fn on_boot(&self, _args: Vec<String>) -> Result<(), Error> {
        info!("OnBoot logic");
        
        // TODO:
        //  - Put in loop
        //  - Potentially put each subsystem in own thread
        //if let Err(error) = eps::get_telem() {
        //    error!("Error while fetching EPS telemetry: {:?}", error);
        //}
        
        if let Err(error) = mai400::get_telem() {
            error!("Error while fetching MAI-400 telemetry: {:?}", error);
        }

        if let Err(error) = nsl::get_telem() {
            error!("Error while fetching NSL radio telemetry: {:?}", error);
        }

        if let Err(error) = oem6::get_telem() {
            error!("Error while fetching OEM6 telemetry: {:?}", error);
        }

        if let Err(error) = sup_mcu::get_telem() {
            error!("Error while fetching Sup MCU telemetry: {:?}", error);
        }
        
        Ok(())
    }

    fn on_command(&self, _args: Vec<String>) -> Result<(), Error> {
        info!("OnCommand logic called");

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let app = MyApp;
    app_main!(&app)?;

    Ok(())
}
