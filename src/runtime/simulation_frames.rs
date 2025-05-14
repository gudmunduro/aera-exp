use std::process::exit;
use crate::types::EntityVariableKey;
use crate::types::runtime::System;
use crate::types::value::Value;

pub fn set_simulation_frame(frame: u64, system: &mut System) {
    fn insert_sift_features(active_features: &[usize], entity: &str, system: &mut System) {
        for i in active_features {
            system.current_state.variables.insert(EntityVariableKey::new(entity, &format!("sift{i}")), Value::ConstantNumber(1.0));
        }
    }

    if frame == 0 {
        system.current_state.variables.insert(EntityVariableKey::new("co3", "color"), Value::Vec(vec![Value::Number(3.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "position"), Value::Vec(vec![
            Value::UncertainNumber(194.0, 10.0),
            Value::UncertainNumber(131.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(200.00112914936733, 0.1),
            Value::UncertainNumber(0.0006397851588985631, 0.1),
            Value::UncertainNumber(0.00041969100129790604, 0.1),
            Value::UncertainNumber(180.00018310546875, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(122.0, 10.0),
            Value::UncertainNumber(123.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(237.00112914936733, 10.0),
            Value::UncertainNumber(4.2559589340950685, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(296.00112914936733, 10.0),
            Value::UncertainNumber(-11.914253831862377, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(141.0, 10.0),
            Value::UncertainNumber(64.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(229.00112914936733, 10.0),
            Value::UncertainNumber(-57.020636810585785, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        insert_sift_features(&[8, 6, 5, 4, 7, 9, 10, 11, 12], "co3", system);
        insert_sift_features(&[6, 5, 7, 4], "co1", system);
        insert_sift_features(&[3, 2, 1], "co2", system);
    }
    else if frame == 1 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(173.0, 10.0),
            Value::UncertainNumber(111.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(289.0059241176081, 10.0),
            Value::UncertainNumber(-9.147577870856974, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "color"), Value::Vec(vec![Value::Number(3.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "position"), Value::Vec(vec![
            Value::UncertainNumber(221.0, 10.0),
            Value::UncertainNumber(172.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(228.00592411760812, 10.0),
            Value::UncertainNumber(-49.998641700644214, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(154.0, 10.0),
            Value::UncertainNumber(166.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(234.00592411760812, 10.0),
            Value::UncertainNumber(7.02263489510047, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(240.00592411760812, 0.1),
            Value::UncertainNumber(30.00135829935579, 0.1),
            Value::UncertainNumber(0.0005472406046465039, 0.1),
            Value::UncertainNumber(179.99229431152344, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));
        insert_sift_features(&[22, 13, 18, 15, 19, 16, 20, 21, 14, 17], "co2", system);
        insert_sift_features(&[25, 7, 29, 24, 27, 30, 26, 31, 28, 23, 11, 9], "co3", system);
        insert_sift_features(&[38, 11, 33, 37, 36, 35, 34], "co1", system);
    }
    else if frame == 2 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(162.0, 10.0),
            Value::UncertainNumber(120.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(280.00441002220657, 10.0),
            Value::UncertainNumber(0.21393499098006785, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "color"), Value::Vec(vec![Value::Number(3.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "position"), Value::Vec(vec![
            Value::UncertainNumber(221.0, 10.0),
            Value::UncertainNumber(172.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(228.00441002220657, 10.0),
            Value::UncertainNumber(-49.99883096646674, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(154.0, 10.0),
            Value::UncertainNumber(166.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(240.00441002220657, 10.0),
            Value::UncertainNumber(30.00116903353326, 10.0),
            Value::UncertainNumber(-0.0010502763325348496, 10.0),
            Value::UncertainNumber(179.9895477294922, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(240.00441002220657, 0.1),
            Value::UncertainNumber(30.00116903353326, 0.1),
            Value::UncertainNumber(-0.0010502763325348496, 0.1),
            Value::UncertainNumber(179.9895477294922, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![Value::EntityId("co1".to_string())]));
        insert_sift_features(&[17, 32, 39, 18, 16, 13, 40], "co2", system);
        insert_sift_features(&[9, 7, 30, 11, 26, 27, 29, 24, 25, 31, 44, 23], "co3", system);
        insert_sift_features(&[34, 11, 33, 37, 38, 35, 36], "co1", system);
    }
    else if frame == 3 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(87.0, 10.0),
            Value::UncertainNumber(109.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(291.00321158766087, 10.0),
            Value::UncertainNumber(-15.955730540969093, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "color"), Value::Vec(vec![Value::Number(3.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "position"), Value::Vec(vec![
            Value::UncertainNumber(141.0, 10.0),
            Value::UncertainNumber(164.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(236.0032115876609, 10.0),
            Value::UncertainNumber(-61.91317734947973, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(154.0, 10.0),
            Value::UncertainNumber(166.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(240.0032115876609, 10.0),
            Value::UncertainNumber(-49.998283732458454, 10.0),
            Value::UncertainNumber(-0.0009094531997106969, 10.0),
            Value::UncertainNumber(179.9876251220703, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(240.0032115876609, 0.1),
            Value::UncertainNumber(-49.998283732458454, 0.1),
            Value::UncertainNumber(-0.0009094531997106969, 0.1),
            Value::UncertainNumber(179.9876251220703, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![Value::EntityId("co1".to_string())]));
        insert_sift_features(&[34, 36, 37, 38, 11, 35, 33], "co1", system);
        insert_sift_features(&[33, 37, 48, 36], "co3", system);
    }
    else if frame == 4 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(87.0, 10.0),
            Value::UncertainNumber(109.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(291.001614979514, 10.0),
            Value::UncertainNumber(-15.952994842880706, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(142.0, 10.0),
            Value::UncertainNumber(171.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(229.00161497951396, 10.0),
            Value::UncertainNumber(-62.761505481178574, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(240.00161497951396, 0.1),
            Value::UncertainNumber(-49.99554803437007, 0.1),
            Value::UncertainNumber(-0.002485763980075717, 0.1),
            Value::UncertainNumber(179.9799346923828, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));
        insert_sift_features(&[35, 51, 50, 36, 48], "co1", system);
    }
    else if frame == 5 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(139.0, 10.0),
            Value::UncertainNumber(176.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(283.9997863017406, 10.0),
            Value::UncertainNumber(-10.206048256466433, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(299.9997863017406, 0.1),
            Value::UncertainNumber(0.006717700980374541, 0.1),
            Value::UncertainNumber(-0.003765871748328209, 0.1),
            Value::UncertainNumber(179.96832275390625, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));
    }
    else if frame == 6 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(139.0, 10.0),
            Value::UncertainNumber(176.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(299.99816884707616, 10.0),
            Value::UncertainNumber(-0.007677331702382827, 10.0),
            Value::UncertainNumber(0.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(299.99816884707616, 0.1),
            Value::UncertainNumber(0.007677331702382827, 0.1),
            Value::UncertainNumber(-0.004854487255215645, 0.1),
            Value::UncertainNumber(179.96575927734375, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![Value::EntityId("co1".to_string())]));
    }
    else if frame == 7 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(139.0, 10.0),
            Value::UncertainNumber(176.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(239.99684729560965, 10.0),
            Value::UncertainNumber(-49.9913495804623, 10.0),
            Value::UncertainNumber(0.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(142.0, 10.0),
            Value::UncertainNumber(171.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(239.99684729560965, 10.0),
            Value::UncertainNumber(-49.9913495804623, 10.0),
            Value::UncertainNumber(-0.0038841667119413614, 10.0),
            Value::UncertainNumber(179.96144104003906, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(239.99684729560965, 0.1),
            Value::UncertainNumber(-49.9913495804623, 0.1),
            Value::UncertainNumber(-0.0038841667119413614, 0.1),
            Value::UncertainNumber(179.96144104003906, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![Value::EntityId("co1".to_string())]));
        insert_sift_features(&[35, 48, 52, 50, 53, 51, 36], "co1", system);
    }
    else if frame == 900 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(139.0, 10.0),
            Value::UncertainNumber(176.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(239.99684729560965, 10.0),
            Value::UncertainNumber(-49.9913495804623, 10.0),
            Value::UncertainNumber(0.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(142.0, 10.0),
            Value::UncertainNumber(171.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(150.99684729560965, 10.0),
            Value::UncertainNumber(10.75730702727081, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(239.99684729560965, 0.1),
            Value::UncertainNumber(-49.9913495804623, 0.1),
            Value::UncertainNumber(-0.0038841667119413614, 0.1),
            Value::UncertainNumber(179.96144104003906, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));
        insert_sift_features(&[35, 48, 52, 50, 53, 51, 36], "co1", system);
    }
    else if frame == 8 {
        system.current_state.variables.insert(EntityVariableKey::new("co3", "color"), Value::Vec(vec![Value::Number(3.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "position"), Value::Vec(vec![
            Value::UncertainNumber(194.0, 10.0),
            Value::UncertainNumber(131.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(200.00112914936733, 0.1),
            Value::UncertainNumber(0.0006397851588985631, 0.1),
            Value::UncertainNumber(0.00041969100129790604, 0.1),
            Value::UncertainNumber(180.00018310546875, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(122.0, 10.0),
            Value::UncertainNumber(123.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(237.00112914936733, 10.0),
            Value::UncertainNumber(4.2559589340950685, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(296.00112914936733, 10.0),
            Value::UncertainNumber(-11.914253831862377, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(141.0, 10.0),
            Value::UncertainNumber(64.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(229.00112914936733, 10.0),
            Value::UncertainNumber(-57.020636810585785, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        insert_sift_features(&[8, 6, 5, 4, 7, 9, 10, 11, 12], "co3", system);
        insert_sift_features(&[6, 5, 7, 4], "co1", system);
        insert_sift_features(&[3, 2, 1], "co2", system);
    }
    else if frame == 9 {
        exit(0);
    }
    else if frame == 9 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(173.0, 10.0),
            Value::UncertainNumber(111.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(289.0059241176081, 10.0),
            Value::UncertainNumber(-9.147577870856974, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "color"), Value::Vec(vec![Value::Number(3.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "position"), Value::Vec(vec![
            Value::UncertainNumber(221.0, 10.0),
            Value::UncertainNumber(172.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(228.00592411760812, 10.0),
            Value::UncertainNumber(-49.998641700644214, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(154.0, 10.0),
            Value::UncertainNumber(166.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(234.00592411760812, 10.0),
            Value::UncertainNumber(7.02263489510047, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(240.00592411760812, 0.1),
            Value::UncertainNumber(30.00135829935579, 0.1),
            Value::UncertainNumber(0.0005472406046465039, 0.1),
            Value::UncertainNumber(179.99229431152344, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));
        insert_sift_features(&[22, 13, 18, 15, 19, 16, 20, 21, 14, 17], "co2", system);
        insert_sift_features(&[25, 7, 29, 24, 27, 30, 26, 31, 28, 23, 11, 9], "co3", system);
        insert_sift_features(&[38, 11, 33, 37, 36, 35, 34], "co1", system);
    }
    else if frame == 10 {
        system.current_state.variables.insert(EntityVariableKey::new("co2", "color"), Value::Vec(vec![Value::Number(2.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "position"), Value::Vec(vec![
            Value::UncertainNumber(162.0, 10.0),
            Value::UncertainNumber(120.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(280.00441002220657, 10.0),
            Value::UncertainNumber(0.21393499098006785, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co2", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "color"), Value::Vec(vec![Value::Number(3.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "position"), Value::Vec(vec![
            Value::UncertainNumber(221.0, 10.0),
            Value::UncertainNumber(172.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(228.00441002220657, 10.0),
            Value::UncertainNumber(-49.99883096646674, 10.0),
            Value::UncertainNumber(-100.0, 10.0),
            Value::UncertainNumber(180.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co3", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "color"), Value::Vec(vec![Value::Number(1.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "position"), Value::Vec(vec![
            Value::UncertainNumber(154.0, 10.0),
            Value::UncertainNumber(166.0, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
            Value::UncertainNumber(240.00441002220657, 10.0),
            Value::UncertainNumber(30.00116903353326, 10.0),
            Value::UncertainNumber(-0.0010502763325348496, 10.0),
            Value::UncertainNumber(179.9895477294922, 10.0)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("co1", "obj_type"), Value::Vec(vec![Value::Number(0.0)]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
            Value::UncertainNumber(240.00441002220657, 0.1),
            Value::UncertainNumber(30.00116903353326, 0.1),
            Value::UncertainNumber(-0.0010502763325348496, 0.1),
            Value::UncertainNumber(179.9895477294922, 0.1)
        ]));
        system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![Value::EntityId("co1".to_string())]));
        insert_sift_features(&[17, 32, 39, 18, 16, 13, 40], "co2", system);
        insert_sift_features(&[9, 7, 30, 11, 26, 27, 29, 24, 25, 31, 44, 23], "co3", system);
        insert_sift_features(&[34, 11, 33, 37, 38, 35, 36], "co1", system);
    }
}