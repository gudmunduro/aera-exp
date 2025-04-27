pub mod pattern_matching;
mod runtime_main;
mod seeds;
pub mod simulation;
pub mod utils;

use crate::interfaces::tcp_interface::TcpInterface;
use crate::runtime::runtime_main::run_aera;
use std::process::exit;
use std::sync::{Arc, Mutex};

pub fn run_demo() {
    run_aera(
        seeds::setup_bindings_seed,
        |_system| {},
        |cmd| {
            log::debug!("Command to execute next {cmd}");
            exit(0);
        },
    );
}

#[allow(unused)]
pub fn run_with_tcp() {
    let tcp_receive_interface = Arc::new(Mutex::new(
        TcpInterface::connect().expect("Failed to connect to controller with TCP"),
    ));
    let tcp_send_interface = tcp_receive_interface.clone();

    run_aera(
        seeds::robot_advanced_move::setup_robot_advanced_seed,
        |system| {
            let tcp_variables = tcp_receive_interface.lock().unwrap().update_variables();
            system.current_state.variables = tcp_variables;
        },
        |cmd| {
            tcp_send_interface
                .lock()
                .unwrap()
                .execute_command(&cmd)
                .expect("Failed to execute command with TCP");
        },
    );
}
