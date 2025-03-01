use std::collections::HashMap;
use crate::types::cst::{Cst, ICst};
use crate::types::{Command, EntityDeclaration, EntityPatternValue, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::functions::Function;
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{PatternItem};
use crate::types::runtime::System;
use crate::types::value::Value;

pub fn setup_hand_grab_sphere_seed(system: &mut System) {
    system.create_entity("h", "hand");
    system.create_entity("b_0", "box");
    system.create_entity("b_1", "box");
    system.create_entity("b_2", "box");

    system.csts.insert(
        "S0".to_string(),
        Cst {
            cst_id: "S0".to_string(),
            facts: vec![Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("p".to_string()),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            }],
            entities: vec![EntityDeclaration::new("h", "hand")],
        },
    );

    system.models.insert(
        "mdl_move_req".to_string(),
        Mdl {
            model_id: "mdl_move_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S0".to_string(),
                    params: vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "mdl_move".to_string(),
                    params: vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("np".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: HashMap::new(),
            backward_computed: HashMap::new(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "mdl_move".to_string(),
        Mdl {
            model_id: "mdl_move".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command {
                    name: "move".to_string(),
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![PatternItem::Binding("dp".to_string())],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("np".to_string()),
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [(
                "dp".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("np".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                ),
            )]
                .into(),
            backward_computed: [].into(),
            confidence: 1.0,
        },
    );

    // Grab cube

    system.csts.insert(
        "S2".to_string(),
        Cst {
            cst_id: "S2".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("b".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::EntityId("h".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
            ],
            entities: vec![EntityDeclaration::new("b", "box")],
        },
    );

    system.models.insert(
        "M_grab_req".to_string(),
        Mdl {
            model_id: "M_grab_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S2".to_string(),
                    params: vec![
                        PatternItem::Binding("b".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "M_grab".to_string(),
                    params: vec![
                        PatternItem::Binding("b".to_string())
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: HashMap::new(),
            backward_computed: HashMap::new(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "M_grab".to_string(),
        Mdl {
            model_id: "M_grab".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command {
                    name: "grab".to_string(),
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    params: vec![],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Binding("b".to_string()),
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [].into(),
            backward_computed: [].into(),
            confidence: 1.0,
        },
    );

    // Release cube

    system.csts.insert(
        "S_holding".to_string(),
        Cst {
            cst_id: "S_holding".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("b".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::EntityId("h".to_string()),
                        var_name: "holding".to_string(),
                        value: PatternItem::Binding("b".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
            ],
            entities: vec![EntityDeclaration::new("b", "box")],
        },
    );

    system.models.insert(
        "M_release_req".to_string(),
        Mdl {
            model_id: "M_release_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S_holding".to_string(),
                    params: vec![
                        PatternItem::Binding("b".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "M_release".to_string(),
                    params: vec![],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: HashMap::new(),
            backward_computed: HashMap::new(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "M_release".to_string(),
        Mdl {
            model_id: "M_release".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command {
                    name: "release".to_string(),
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    params: vec![],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Value(Value::Vec(vec![])),
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [].into(),
            backward_computed: [].into(),
            confidence: 1.0,
        },
    );

    // Move while holding the cube moves the cube

    system.models.insert(
        "M_move_cube_req".to_string(),
        Mdl {
            model_id: "M_move_cube_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S_holding".to_string(),
                    params: vec![
                        PatternItem::Binding("b".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "M_move_cube".to_string(),
                    params: vec![
                        PatternItem::Binding("np".to_string()),
                        PatternItem::Binding("p".to_string()),
                        PatternItem::Binding("b".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: HashMap::new(),
            backward_computed: HashMap::new(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "M_move_cube".to_string(),
        Mdl {
            model_id: "M_move_cube".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command {
                    name: "move".to_string(),
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    params: vec![PatternItem::Binding("dp".to_string())],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("b".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("np".to_string()),
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [(
                "dp".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("np".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                ),
            )].into(),
            backward_computed: [].into(),
            confidence: 1.0,
        },
    );
}