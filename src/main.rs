extern crate core;

use log::error;

use crate::app::App;
use crate::input::utils::get_input_data;
use crate::terminal::init_terminal_app;

mod input;
mod asn1_der;
mod app;
mod terminal;
mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _log2 = log2::open("log.txt").start();

    let input= match get_input_data() {
        Ok(input) => {
            input
        }
        Err(err) => {
            error!("input error: {:?}", err);
            return Err(Box::new(err))
        }
    };

    let app = match App::new(input) {
        Ok(app) => app,
        Err(err) => {
            error!("app error: {:?}", err);
            return Err(Box::new(err))
        }
    };

    match init_terminal_app(app) {
        Ok(_) => Ok(()),
        Err(err) => {
            error!("terminal error: {:?}", err);
            return Err(err)
        }
    }
}