pub mod learning;
pub mod pattern_matching;
mod runtime_main;
mod seeds;
pub mod simulation;
pub mod utils;

use crate::interfaces::tcp_interface::TcpInterface;
use crate::runtime::runtime_main::run_aera;
use crate::types::value::Value;
use crate::types::EntityVariableKey;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tap::Pipe;
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
    fn insert_sift_features(active_features: &[usize], entity: &str, system: &mut System) {
        for i in active_features {
            system.current_state.variables.insert(EntityVariableKey::new(entity, &format!("sift{i}")), Value::ConstantNumber(1.0));
        }
    }

    run_aera(
        seeds::robot_sift_learn::setup_robot_sift_learn_seed,
        |system| {
            if system.current_state.variables.is_empty() {
                system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::UncertainNumber(200.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(180.0, 0.1)]));
                system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));

                system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![Value::UncertainNumber(200.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(180.0, 5.0)]));
                system.current_state.variables.insert(EntityVariableKey::new("co1", "pos"), Value::Vec(vec![Value::UncertainNumber(136.0, 5.0), Value::UncertainNumber(126.0, 5.0)]));
                system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
                insert_sift_features(&[0], "co1", system);
            }
        },
        |cmd, system| match &cmd.name[..] {
            "move" | "grab" | "release" => {
                system.current_state.variables.clear();
                let frame = system.current_state.time.pipe_ref(|t| match t { SystemTime::Exact(t) => *t / 100, _ => panic!() });
                log::debug!("Got to frame {frame}");
                if frame == 0 {
                    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::UncertainNumber(240.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(180.0, 0.1)]));
                    system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));

                    system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![Value::UncertainNumber(240.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(-90.0, 5.0), Value::UncertainNumber(180.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "pos"), Value::Vec(vec![Value::UncertainNumber(133.0, 5.0), Value::UncertainNumber(170.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
                    insert_sift_features(&[0], "co1", system);
                }
                else if frame == 1 {
                    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::UncertainNumber(240.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(180.0, 0.1)]));
                    system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));

                    system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![Value::UncertainNumber(240.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(-90.0, 5.0), Value::UncertainNumber(180.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "pos"), Value::Vec(vec![Value::UncertainNumber(143.0, 5.0), Value::UncertainNumber(159.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
                    insert_sift_features(&[1, 2, 3, 4, 5], "co1", system);
                }
                else if frame == 2 {
                    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::UncertainNumber(240.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(180.0, 0.1)]));
                    system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![Value::EntityId("co1".to_string())]));

                    system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![Value::UncertainNumber(240.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(180.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "pos"), Value::Vec(vec![Value::UncertainNumber(143.0, 5.0), Value::UncertainNumber(159.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
                    insert_sift_features(&[1, 2, 3, 4, 5], "co1", system);
                }
                else if frame == 3 {
                    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::UncertainNumber(240.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(180.0, 0.1)]));
                    system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));

                    system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![Value::UncertainNumber(240.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(-90.0, 5.0), Value::UncertainNumber(180.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "pos"), Value::Vec(vec![Value::UncertainNumber(143.0, 5.0), Value::UncertainNumber(159.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
                    insert_sift_features(&[1, 2, 3, 4], "co1", system);
                }
                else if frame == 4 {
                    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::UncertainNumber(240.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(180.0, 0.1)]));
                    system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![Value::EntityId("co1".to_string())]));

                    system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![Value::UncertainNumber(240.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(180.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "pos"), Value::Vec(vec![Value::UncertainNumber(143.0, 5.0), Value::UncertainNumber(159.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
                    insert_sift_features(&[1, 2, 3, 4], "co1", system);
                }
                else if frame == 5 {
                    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::UncertainNumber(240.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(180.0, 0.1)]));
                    system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));

                    system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![Value::UncertainNumber(240.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(-90.0, 5.0), Value::UncertainNumber(180.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "pos"), Value::Vec(vec![Value::UncertainNumber(143.0, 5.0), Value::UncertainNumber(159.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
                    insert_sift_features(&[1, 2, 3, 4, 9, 13], "co1", system);
                }
                else if frame == 6 {
                    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::UncertainNumber(240.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(180.0, 0.1)]));
                    system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));

                    system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![Value::UncertainNumber(240.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(-90.0, 5.0), Value::UncertainNumber(180.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "pos"), Value::Vec(vec![Value::UncertainNumber(143.0, 5.0), Value::UncertainNumber(159.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
                    insert_sift_features(&[1, 2, 3, 4, 9, 13], "co1", system);
                }
                else if frame == 7 {
                    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::UncertainNumber(240.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(180.0, 0.1)]));
                    system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));

                    system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![Value::UncertainNumber(240.0, 5.0), Value::UncertainNumber(0.0, 5.0), Value::UncertainNumber(-90.0, 5.0), Value::UncertainNumber(180.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "pos"), Value::Vec(vec![Value::UncertainNumber(143.0, 5.0), Value::UncertainNumber(159.0, 5.0)]));
                    system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
                    insert_sift_features(&[1, 2, 3, 4, 9, 13], "co1", system);
                }
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
