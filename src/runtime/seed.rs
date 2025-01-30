use crate::types::cst::{Cst, ICst};
use crate::types::functions::Function;
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{PatternItem, PatternValue};
use crate::types::runtime::{RuntimeValue, System};
use crate::types::{
    Command, EntityDeclaration, EntityPatternValue, EntityVariableKey, Fact, MkVal,
    TimePatternRange, TimePatternValue,
};
use std::collections::HashMap;

pub fn setup_bindings_seed(system: &mut System) {
    system.create_entity("h", "hand");
    system.create_entity("o", "object");

    system.csts.insert(
        "cst_pos".to_string(),
        Cst {
            cst_id: "cst_pos".to_string(),
            facts: vec![Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::Binding("hb".to_string()),
                    var_name: "pos".to_string(),
                    value: PatternItem::Binding("p".to_string()),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            }],
            entities: vec![EntityDeclaration::new("hb", "hand")],
        },
    );

    system.models.insert(
        "mdl_move_req".to_string(),
        Mdl {
            model_id: "mdl_move_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "cst_pos".to_string(),
                    pattern: vec![
                        PatternItem::Binding("hb".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "mdl_move".to_string(),
                    params: vec![
                        PatternItem::Binding("hb".to_string()),
                        PatternItem::Binding("cp".to_string()),
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
                    entity_id: EntityPatternValue::Binding("hb".to_string()),
                    params: vec![PatternItem::Binding("dp".to_string())],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("hb".to_string()),
                    var_name: "pos".to_string(),
                    value: PatternItem::Binding("cp".to_string()),
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [(
                "dp".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("cp".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                ),
            )]
            .into(),
            backward_computed: [].into(),
            confidence: 1.0,
        },
    );

    system.csts.insert(
        "cst_obj".to_string(),
        Cst {
            cst_id: "cst_obj".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("hb".to_string()),
                        var_name: "pos".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::EntityId("o".to_string()),
                        var_name: "pos".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
            ],
            entities: vec![EntityDeclaration::new("hb", "hand")],
        },
    );

    system.models.insert(
        "mdl_push_req".to_string(),
        Mdl {
            model_id: "mdl_push_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "cst_obj".to_string(),
                    pattern: vec![
                        PatternItem::Binding("hb".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "mdl_push".to_string(),
                    params: vec![PatternItem::Binding("p".to_string())],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: HashMap::new(),
            backward_computed: HashMap::new(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "mdl_push".to_string(),
        Mdl {
            model_id: "mdl_push".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command {
                    name: "push".to_string(),
                    entity_id: EntityPatternValue::EntityId("o".to_string()),
                    params: vec![],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::EntityId("o".to_string()),
                    var_name: "pos".to_string(),
                    value: PatternItem::Binding("np".to_string()),
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [(
                "np".to_string(),
                Function::Add(
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                    Box::new(Function::Value(PatternItem::Value(PatternValue::List(
                        vec![PatternValue::Number(0.0), PatternValue::Number(1.0)],
                    )))),
                ),
            )]
            .into(),
            backward_computed: [(
                "p".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("np".to_string()))),
                    Box::new(Function::Value(PatternItem::Value(PatternValue::List(
                        vec![PatternValue::Number(0.0), PatternValue::Number(1.0)],
                    )))),
                ),
            )]
            .into(),
            confidence: 1.0,
        },
    );

    system.current_state.variables.insert(
        EntityVariableKey::new("h", "pos"),
        RuntimeValue::List(vec![RuntimeValue::Number(1.0), RuntimeValue::Number(1.0)]),
    );
    system.current_state.variables.insert(
        EntityVariableKey::new("o", "pos"),
        RuntimeValue::List(vec![RuntimeValue::Number(5.0), RuntimeValue::Number(5.0)]),
    );
}

pub fn setup_simple_seed(system: &mut System) {
    system.create_entity("h", "hand");
    system.create_entity("o", "object");

    system.csts.insert(
        "cst_pos".to_string(),
        Cst {
            cst_id: "cst_pos".to_string(),
            facts: vec![Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("p".to_string()),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            }],
            entities: vec![],
        },
    );

    system.models.insert(
        "mdl_move_req".to_string(),
        Mdl {
            model_id: "mdl_move_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "cst_pos".to_string(),
                    pattern: vec![PatternItem::Binding("p".to_string())],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "mdl_move".to_string(),
                    params: vec![
                        PatternItem::Binding("cp".to_string()),
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
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    params: vec![PatternItem::Binding("dp".to_string())],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("cp".to_string()),
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [(
                "dp".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("cp".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                ),
            )]
            .into(),
            backward_computed: [].into(),
            confidence: 1.0,
        },
    );

    system.csts.insert(
        "cst_obj".to_string(),
        Cst {
            cst_id: "cst_obj".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::EntityId("h".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::EntityId("o".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
            ],
            entities: Vec::new(),
        },
    );

    system.models.insert(
        "mdl_push_req".to_string(),
        Mdl {
            model_id: "mdl_push_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "cst_obj".to_string(),
                    pattern: vec![PatternItem::Binding("p".to_string())],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "mdl_push".to_string(),
                    params: vec![PatternItem::Binding("p".to_string())],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: HashMap::new(),
            backward_computed: HashMap::new(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "mdl_push".to_string(),
        Mdl {
            model_id: "mdl_push".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command {
                    name: "push".to_string(),
                    entity_id: EntityPatternValue::EntityId("o".to_string()),
                    params: vec![],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::EntityId("o".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("np".to_string()),
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [(
                "np".to_string(),
                Function::Add(
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                    Box::new(Function::Value(PatternItem::Value(PatternValue::Number(
                        1.0,
                    )))),
                ),
            )]
            .into(),
            backward_computed: [(
                "p".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("np".to_string()))),
                    Box::new(Function::Value(PatternItem::Value(PatternValue::Number(
                        1.0,
                    )))),
                ),
            )]
            .into(),
            confidence: 1.0,
        },
    );

    system.current_state.variables.insert(
        EntityVariableKey::new("h", "position"),
        RuntimeValue::Number(1.0),
    );
    system.current_state.variables.insert(
        EntityVariableKey::new("o", "position"),
        RuntimeValue::Number(5.0),
    );
}
