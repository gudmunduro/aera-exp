use crate::types::{EntityPatternValue, EntityVariableKey, Fact, MkVal, TimePatternRange};
use crate::types::pattern::PatternItem;
use crate::types::runtime::{RuntimeCommand, System};
use crate::types::value::Value;

#[allow(unused)]
pub fn setup_hand_grab_sphere_learn_seed(system: &mut System) {
    system.create_entity("h", "hand");
    system.create_entity("c", "cube");
    system.create_entity("s", "sphere");

    system.current_state.variables.insert(
        EntityVariableKey::new("h", "position"),
        Value::Number(20.0),
    );
    system.current_state.variables.insert(
        EntityVariableKey::new("h", "holding"),
        Value::Vec(vec![]),
    );
    system.current_state.variables.insert(
        EntityVariableKey::new("c", "position"),
        Value::Number(10.0),
    );
    system.current_state.variables.insert(
        EntityVariableKey::new("s", "position"),
        Value::Number(5.0),
    );

    system.babble_command.push(RuntimeCommand::new("move".to_string(), "h".to_string(), vec![Value::Number(-5.0)]));
    system.babble_command.push(RuntimeCommand::new("move".to_string(), "h".to_string(), vec![Value::Number(-10.0)]));
    system.babble_command.push(RuntimeCommand::new("grab".to_string(), "h".to_string(), vec![]));
    system.babble_command.push(RuntimeCommand::new("release".to_string(), "h".to_string(), vec![]));
    system.babble_command.push(RuntimeCommand::new("grab".to_string(), "h".to_string(), vec![]));
    system.babble_command.push(RuntimeCommand::new("grab".to_string(), "h".to_string(), vec![]));
    system.babble_command.push(RuntimeCommand::new("move".to_string(), "h".to_string(), vec![Value::Number(-5.0)]));

    system.goals = vec![
        vec![
            Fact::new(MkVal {
                entity_id: EntityPatternValue::EntityId("s".to_string()),
                var_name: "position".to_string(),
                value: PatternItem::Value(Value::Number(30.0)),
                assumption: false,
            }, TimePatternRange::wildcard())
        ],
        vec![
            Fact::new(MkVal {
                entity_id: EntityPatternValue::EntityId("h".to_string()),
                var_name: "holding".to_string(),
                value: PatternItem::Vec(vec![]),
                assumption: false,
            }, TimePatternRange::wildcard())
        ],
        vec![
            Fact::new(MkVal {
                entity_id: EntityPatternValue::EntityId("h".to_string()),
                var_name: "position".to_string(),
                value: PatternItem::Value(Value::Number(20.0)),
                assumption: false,
            }, TimePatternRange::wildcard())
        ],
        vec![
            Fact::new(MkVal {
                entity_id: EntityPatternValue::EntityId("s".to_string()),
                var_name: "position".to_string(),
                value: PatternItem::Value(Value::Number(20.0)),
                assumption: false,
            }, TimePatternRange::wildcard())
        ],
    ]
}