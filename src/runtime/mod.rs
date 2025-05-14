pub mod learning;
pub mod pattern_matching;
mod runtime_main;
mod seeds;
pub mod simulation;
pub mod utils;
mod simulation_frames;

use crate::interfaces::tcp_interface::TcpInterface;
use crate::runtime::runtime_main::run_aera;
use crate::types::value::Value;
use crate::types::EntityVariableKey;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tap::Pipe;
use crate::runtime::simulation_frames::set_simulation_frame;
use crate::types::runtime::{System, SystemTime};

pub fn run_demo() {
    run_aera(
        seeds::setup_bindings_seed,
        |_system| {},
        |cmd, _system| {
            log::debug!("Command to execute next {cmd}");
            exit(0);
        },
    );
}

pub fn run_hand_grab_sphere_learn_demo() {
    run_aera(
        seeds::hand_grab_sphere_learn::setup_hand_grab_sphere_learn_seed,
        |_system| {},
        |cmd, system| match &cmd.name[..] {
            "move" => {
                let Value::Number(move_by) = &cmd.params[0] else {
                    panic!("Invalid parameters supplied to move command");
                };
                match system
                    .current_state
                    .variables
                    .get_mut(&EntityVariableKey::new("h", "position"))
                    .unwrap()
                {
                    Value::Number(pos) => {
                        *pos += move_by;
                    }
                    _ => {}
                }
                if let Some(Value::EntityId(holding)) = system
                    .current_state
                    .variables
                    .get(&EntityVariableKey::new("h", "holding"))
                    .unwrap()
                    .as_vec()
                    .iter()
                    .next()
                {
                    match system
                        .current_state
                        .variables
                        .get_mut(&EntityVariableKey::new(holding, "position"))
                        .unwrap()
                    {
                        Value::Number(pos) => {
                            *pos += move_by;
                        }
                        _ => {}
                    }
                }
            }
            "grab" => {
                let current_pos = system
                    .current_state
                    .variables
                    .get(&EntityVariableKey::new("h", "position"))
                    .unwrap()
                    .clone();
                let cube_pos = system
                    .current_state
                    .variables
                    .get(&EntityVariableKey::new("c", "position"))
                    .unwrap()
                    .clone();
                let sphere_pos = system
                    .current_state
                    .variables
                    .get(&EntityVariableKey::new("s", "position"))
                    .unwrap()
                    .clone();

                let holding = system
                    .current_state
                    .variables
                    .get_mut(&EntityVariableKey::new("h", "holding"))
                    .unwrap();

                if current_pos == cube_pos {
                    *holding = Value::Vec(vec![Value::EntityId("c".to_string())]);
                } else if current_pos == sphere_pos {
                    *holding = Value::Vec(vec![Value::EntityId("s".to_string())]);
                }
            }
            "release" => {
                let holding = system
                    .current_state
                    .variables
                    .get_mut(&EntityVariableKey::new("h", "holding"))
                    .unwrap();
                *holding = Value::Vec(vec![]);
            }
            _ => {
                std::thread::sleep(Duration::from_secs(5));
            }
        },
    );
}

pub fn run_simulated_robot_learn_demo() {
    run_aera(
        seeds::robot_sift_learn::setup_robot_sift_learn_seed,
        |system| {
            if system.current_state.variables.is_empty() && !system.babble_command.is_empty() {
                set_simulation_frame(0, system);
            }
        },
        |cmd, system| match &cmd.name[..] {
            "move" | "grab" | "release" => {
                system.current_state.variables.clear();
                let frame = system.current_state.time.pipe_ref(|t| match t { SystemTime::Exact(t) => *t / 100, _ => panic!() }) + 1;
                log::debug!("Got to frame {frame}");
                set_simulation_frame(frame, system);
            }
            "no_action" => {
                let frame = system.current_state.time.pipe_ref(|t| match t { SystemTime::Exact(t) => *t / 100, _ => panic!() });
                if frame >= 8 {
                    exit(0);
                }
            }
            _ => {}
        }
    )
}

#[allow(unused)]
pub fn run_with_tcp() {
    let tcp_receive_interface = Arc::new(Mutex::new(
        TcpInterface::connect().expect("Failed to connect to controller with TCP"),
    ));
    let tcp_send_interface = tcp_receive_interface.clone();

    run_aera(
        seeds::robot_sift_learn::setup_robot_sift_learn_seed,
        |system| {
            let tcp_variables = tcp_receive_interface.lock().unwrap().update_variables();
            system.current_state.variables = tcp_variables;
        },
        |cmd, _system| {
            tcp_send_interface
                .lock()
                .unwrap()
                .execute_command(&cmd)
                .expect("Failed to execute command with TCP");
        },
    );
}
