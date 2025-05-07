pub mod hand_grab_sphere;
pub mod robot_advanced_move;
pub mod hand_grab_sphere_learn;
pub mod robot_sift_learn;

use crate::types::cst::{Cst, ICst};
use crate::types::{Command, EntityDeclaration, EntityPatternValue, EntityVariableKey, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::functions::Function;
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{PatternItem};
use crate::types::runtime::System;
use crate::types::value::Value;

pub fn setup_bindings_seed(system: &mut System) {
    system.create_entity("h", "hand");
    system.create_entity("o", "object");

    system.csts.insert(
        "cst_pos".to_string(),
        Cst {
            cst_id: "cst_pos".to_string(),
            facts: vec![Fact::new(
                MkVal {
                    entity_id: EntityPatternValue::Binding("hb".to_string()),
                    var_name: "pos".to_string(),
                    value: PatternItem::Binding("p".to_string()),
                    assumption: false,
                },
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            )],
            entities: vec![EntityDeclaration::new("hb", "hand")],
        },
    );

    system.models.insert(
        "mdl_move_req".to_string(),
        Mdl {
            model_id: "mdl_move_req".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "cst_pos".to_string(),
                    params: vec![
                        PatternItem::Binding("hb".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "mdl_move".to_string(),
                    vec![
                        PatternItem::Binding("hb".to_string()),
                        PatternItem::Binding("dp".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "mdl_move".to_string(),
        Mdl {
            model_id: "mdl_move".to_string(),
            left: Fact::new(
                MdlLeftValue::Command(Command {
                    name: "move".to_string(),
                    entity_id: EntityPatternValue::Binding("hb".to_string()),
                    params: vec![PatternItem::Binding("dp".to_string())],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("hb".to_string()),
                    var_name: "pos".to_string(),
                    value: PatternItem::Binding("cp".to_string()),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [(
                "cp".to_string(),
                Function::Add(
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("dp".to_string()))),
                ),
            )]
                .into(),
            backward_computed: [(
                "dp".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("cp".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                ),
            )]
                .into(),
            confidence: 1.0,
        },
    );

    system.csts.insert(
        "cst_obj".to_string(),
        Cst {
            cst_id: "cst_obj".to_string(),
            facts: vec![
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::Binding("hb".to_string()),
                        var_name: "pos".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::EntityId("o".to_string()),
                        var_name: "pos".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
            ],
            entities: vec![EntityDeclaration::new("hb", "hand")],
        },
    );

    system.models.insert(
        "mdl_push_req".to_string(),
        Mdl {
            model_id: "mdl_push_req".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "cst_obj".to_string(),
                    params: vec![
                        PatternItem::Binding("hb".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "mdl_push".to_string(),
                    vec![PatternItem::Binding("p".to_string())],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "mdl_push".to_string(),
        Mdl {
            model_id: "mdl_push".to_string(),
            left: Fact::new(
                MdlLeftValue::Command(Command {
                    name: "push".to_string(),
                    entity_id: EntityPatternValue::EntityId("o".to_string()),
                    params: vec![],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::EntityId("o".to_string()),
                    var_name: "pos".to_string(),
                    value: PatternItem::Binding("np".to_string()),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [(
                "np".to_string(),
                Function::Add(
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                    Box::new(Function::Value(PatternItem::Value(Value::Vec(
                        vec![Value::Number(0.0), Value::Number(1.0)],
                    )))),
                ),
            )]
                .into(),
            backward_computed: [(
                "p".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("np".to_string()))),
                    Box::new(Function::Value(PatternItem::Value(Value::Vec(
                        vec![Value::Number(0.0), Value::Number(1.0)],
                    )))),
                ),
            )]
                .into(),
            confidence: 1.0,
        },
    );

    system.csts.insert(
        "cst_obj_pos".to_string(),
        Cst {
            cst_id: "cst_obj_pos".to_string(),
            facts: vec![
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::EntityId("o".to_string()),
                        var_name: "pos".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
            ],
            entities: vec![],
        },
    );

    system.models.insert(
        "mdl_o_pos_alias".to_string(),
        Mdl {
            model_id: "mdl_o_pos_alias".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "cst_obj_pos".to_string(),
                    params: vec![
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::EntityId("o".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("p".to_string()),
                    assumption: true,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
        },
    );

    system.current_state.variables.insert(
        EntityVariableKey::new("h", "pos"),
        Value::Vec(vec![Value::Number(1.0), Value::Number(1.0)]),
    );
    system.current_state.variables.insert(
        EntityVariableKey::new("o", "pos"),
        Value::Vec(vec![Value::Number(5.0), Value::Number(5.0)]),
    );

    system.goals = vec![
        vec![
            Fact::new(
                MkVal {
                    entity_id: EntityPatternValue::EntityId("o".to_string()),
                    var_name: "pos".to_string(),
                    value: PatternItem::Value(Value::Vec(vec![Value::UncertainNumber(5.0, 0.1), Value::UncertainNumber(7.0, 0.1)])),
                    assumption: false,
                },
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            ),
        ]
    ]
}
