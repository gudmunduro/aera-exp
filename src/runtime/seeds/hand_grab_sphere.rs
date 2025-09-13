use crate::types::cst::{Cst, ICst};
use crate::types::{Command, EntityDeclaration, EntityPatternValue, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::functions::Function;
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{PatternItem};
use crate::types::runtime::System;
use crate::types::value::Value;

#[allow(unused)]
pub fn setup_hand_grab_sphere_seed(system: &mut System) {
    system.create_entity("h", "hand");
    system.create_entity("b_0", "box");
    system.create_entity("b_1", "box");
    system.create_entity("b_2", "box");

    system.csts.insert(
        "S0".to_string(),
        Cst {
            cst_id: "S0".to_string(),
            facts: vec![Fact::new(
                MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("p".to_string()),
                    assumption: false,
                },
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            )],
            entities: vec![EntityDeclaration::new("h", "hand")],
            success_count: 5,
            failure_count: 0,
        },
    );

    system.models.insert(
        "mdl_move_req".to_string(),
        Mdl {
            model_id: "mdl_move_req".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "S0".to_string(),
                    params: vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "mdl_move".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("dp".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            failure_count: 0,
            success_count: 1,
        },
    );

    system.models.insert(
        "mdl_move".to_string(),
        Mdl {
            model_id: "mdl_move".to_string(),
            left: Fact::new(
                MdlLeftValue::Command(Command {
                    name: "move".to_string(),
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![PatternItem::Binding("dp".to_string())],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("np".to_string()),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [(
                "np".to_string(),
                Function::Add(
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("dp".to_string()))),
                ),
            )]
                .into(),
            backward_computed: [(
                "dp".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("np".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                ),
            )]
                .into(),
            failure_count: 0,
            success_count: 1,
        },
    );

    // Grab cube

    system.csts.insert(
        "S2".to_string(),
        Cst {
            cst_id: "S2".to_string(),
            facts: vec![
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::Binding("b".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::EntityId("h".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::EntityId("h".to_string()),
                        var_name: "holding".to_string(),
                        value: PatternItem::Vec(vec![]),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
            ],
            entities: vec![EntityDeclaration::new("b", "box")],
            success_count: 5,
            failure_count: 0,
        },
    );

    system.models.insert(
        "M_grab_req".to_string(),
        Mdl {
            model_id: "M_grab_req".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "S2".to_string(),
                    params: vec![
                        PatternItem::Binding("b".to_string()),
                        PatternItem::Binding("p".to_string()),
                        PatternItem::Binding("h".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "M_grab".to_string(),
                    vec![
                        PatternItem::Binding("b".to_string())
                    ],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            failure_count: 0,
            success_count: 1,
        },
    );

    system.models.insert(
        "M_grab".to_string(),
        Mdl {
            model_id: "M_grab".to_string(),
            left: Fact::new(
                MdlLeftValue::Command(Command {
                    name: "grab".to_string(),
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    params: vec![],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("b".to_string())]),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [].into(),
            backward_computed: [].into(),
            failure_count: 0,
            success_count: 1,
        },
    );

    // Release cube

    system.csts.insert(
        "S_holding".to_string(),
        Cst {
            cst_id: "S_holding".to_string(),
            facts: vec![
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::Binding("b".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::EntityId("h".to_string()),
                        var_name: "holding".to_string(),
                        value: PatternItem::Vec(vec![PatternItem::Binding("b".to_string())]),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
            ],
            entities: vec![EntityDeclaration::new("b", "box")],
            success_count: 5,
            failure_count: 0,
        },
    );

    system.models.insert(
        "M_release_req".to_string(),
        Mdl {
            model_id: "M_release_req".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "S_holding".to_string(),
                    params: vec![
                        PatternItem::Binding("b".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "M_release".to_string(),
                    vec![],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            failure_count: 0,
            success_count: 1,
        },
    );

    system.models.insert(
        "M_release".to_string(),
        Mdl {
            model_id: "M_release".to_string(),
            left: Fact::new(
                MdlLeftValue::Command(Command {
                    name: "release".to_string(),
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    params: vec![],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Vec(vec![]),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [].into(),
            backward_computed: [].into(),
            failure_count: 0,
            success_count: 1,
        },
    );

    // Move while holding the cube moves the cube

    system.models.insert(
        "M_move_cube_req".to_string(),
        Mdl {
            model_id: "M_move_cube_req".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "S_holding".to_string(),
                    params: vec![
                        PatternItem::Binding("b".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "M_move_cube".to_string(),
                    vec![
                        PatternItem::Binding("dp".to_string()),
                        PatternItem::Binding("p".to_string()),
                        PatternItem::Binding("b".to_string()),
                    ],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            failure_count: 0,
            success_count: 1,
        },
    );

    system.models.insert(
        "M_move_cube".to_string(),
        Mdl {
            model_id: "M_move_cube".to_string(),
            left: Fact::new(
                MdlLeftValue::Command(Command {
                    name: "move".to_string(),
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    params: vec![PatternItem::Binding("dp".to_string())],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("b".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("np".to_string()),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [(
                "np".to_string(),
                Function::Add(
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("dp".to_string()))),
                ),
            )].into(),
            backward_computed: [(
                "dp".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("np".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                ),
            )].into(),
            failure_count: 0,
            success_count: 1,
        },
    );

    system.goals = vec![
        vec![
            Fact::new(
                MkVal {
                    entity_id: EntityPatternValue::EntityId("b_0".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Value(Value::Vec(vec![Value::Number(0.0), Value::Number(-0.7), Value::Number(0.0)])),
                    assumption: false,
                },
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            ),
            /*Fact::new(
                MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Vec(vec![]),
                    assumption: false,
                },
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            ),*/
        ],
        vec![
            Fact::new(
                MkVal {
                    entity_id: EntityPatternValue::EntityId("b_1".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Value(Value::Vec(vec![Value::Number(0.0), Value::Number(-1.0), Value::Number(0.0)])),
                    assumption: false,
                },
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            ),
            /*Fact::new(
                MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Value(Value::Vec(vec![Value::EntityId("b_0".to_string())])),
                    assumption: false,
                },
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            ),*/
        ],
        vec![
            Fact::new(
                MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Vec(vec![]),
                    assumption: false,
                },
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            )
        ]
    ];
}