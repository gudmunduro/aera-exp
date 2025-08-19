use std::collections::HashMap;
use std::vec;
use crate::types::cst::{Cst, ICst};
use crate::types::{Command, EntityDeclaration, EntityPatternValue, EntityVariableKey, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::functions::Function;
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::PatternItem;
use crate::types::runtime::{RuntimeCommand, System};
use crate::types::value::Value;

pub fn setup_robot_sift_learn_seed(system: &mut System) {
    system.create_entity("h", "hand");
    system.create_entity("c", "camera");
    system.create_entity("co1", "cam_obj");
    system.create_entity("co2", "cam_obj");
    system.create_entity("co3", "cam_obj");

    // Hand movement

    system.csts.insert(
        "S_move".to_string(),
        Cst {
            cst_id: "S_move".to_string(),
            facts: vec![Fact::new(
                MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("px".to_string()), PatternItem::Binding("py".to_string()), PatternItem::Binding("pz".to_string()), PatternItem::Binding("pw".to_string())]),

                    assumption: false,
                },
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            )],
            entities: vec![EntityDeclaration::new("h", "hand")],
        },
    );

    system.models.insert(
        "mdl_move_req".to_string(),
        Mdl {
            model_id: "mdl_move_req".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "S_move".to_string(),
                    params: vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("pz".to_string()),
                        PatternItem::Binding("pw".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "mdl_move".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("dpx".to_string()),
                        PatternItem::Binding("dpy".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("pz".to_string()),
                        PatternItem::Binding("pw".to_string()),
                    ],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
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
                    params: vec![PatternItem::Vec(vec![
                        PatternItem::Binding("dpx".to_string()),
                        PatternItem::Binding("dpy".to_string()),
                        PatternItem::Value(Value::Number(0.0)),
                        PatternItem::Value(Value::Number(0.0)),
                    ])],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("npx".to_string()), PatternItem::Binding("npy".to_string()), PatternItem::Binding("pz".to_string()), PatternItem::Binding("pw".to_string())]),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [(
                "npx".to_string(),
                Function::Add(
                    Box::new(Function::Value(PatternItem::Binding("px".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("dpx".to_string()))),
                    ),
                ),
                ("npy".to_string(),
                Function::Add(
                    Box::new(Function::Value(PatternItem::Binding("py".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("dpy".to_string()))),
                ),
                ),
            ]
                .into(),
            backward_computed: [(
                    "dpx".to_string(),
                    Function::Sub(
                        Box::new(Function::Value(PatternItem::Binding("npx".to_string()))),
                        Box::new(Function::Value(PatternItem::Binding("px".to_string()))),
                    ),
                ),
                (
                "dpy".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("npy".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("py".to_string()))),
                ),
                ),
            ].into(),
            confidence: 1.0,
            success_count: 1,
        },
    );

    // Grab cube

    system.csts.insert(
        "S1".to_string(),
        Cst {
            cst_id: "S1".to_string(),
            facts: vec![
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::Binding("co".to_string()),
                        var_name: "approximate_pos".to_string(),
                        value: PatternItem::Vec(vec![PatternItem::Binding("px".to_string()), PatternItem::Binding("py".to_string()), PatternItem::Binding("pz".to_string()), PatternItem::Binding("pw".to_string())]),
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
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::EntityId("h".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Vec(vec![PatternItem::Binding("px".to_string()), PatternItem::Binding("py".to_string()), PatternItem::Value(Value::Number(0.0)), PatternItem::Binding("pw".to_string())]),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
            ],
            entities: vec![
                EntityDeclaration::new("h", "hand"),
                EntityDeclaration::new("co", "cam_obj"),
            ],
        },
    );

    system.models.insert(
        "M_grab_req".to_string(),
        Mdl {
            model_id: "M_grab_req".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "S1".to_string(),
                    params: vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("pz".to_string()),
                        PatternItem::Binding("pw".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "M_grab".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("co".to_string()),
                    ],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
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
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("co".to_string())]),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [].into(),
            backward_computed: [].into(),
            confidence: 1.0,
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
                        entity_id: EntityPatternValue::Binding("h".to_string()),
                        var_name: "holding".to_string(),
                        value: PatternItem::Vec(vec![PatternItem::Binding("co".to_string())]),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
                Fact::new(
                    MkVal {
                        entity_id: EntityPatternValue::Binding("co".to_string()),
                        var_name: "approximate_pos".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                        assumption: false,
                    },
                    TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                ),
            ],
            entities: vec![
                EntityDeclaration::new("co", "cam_obj"),
                EntityDeclaration::new("h", "hand"),
            ],
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
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "M_release".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                    ],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
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
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Value(Value::Vec(vec![])),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [].into(),
            backward_computed: [].into(),
            confidence: 1.0,
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
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::IMdl(IMdl::new(
                    "M_move_cube".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("dp".to_string()),
                        PatternItem::Binding("p".to_string()),
                        PatternItem::Binding("co".to_string()),
                    ],
                )),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
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
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![PatternItem::Binding("dp".to_string())],
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            right: Fact::new(
                MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("co".to_string()),
                    var_name: "approximate_pos".to_string(),
                    value: PatternItem::Binding("np".to_string()),
                    assumption: false,
                }),
                TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            ),
            forward_computed: [
                (
                    "np".to_string(),
                    Function::Add(
                        Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                        Box::new(Function::Value(PatternItem::Binding("dp".to_string()))),
                    )
                ),
            ].into(),
            backward_computed: [
                (
                    "dp".to_string(),
                    Function::Sub(
                        Box::new(Function::Value(PatternItem::Binding("np".to_string()))),
                        Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                    )
                ),
            ].into(),
            confidence: 1.0,
            success_count: 1,
        },
    );

    let loaded_models: HashMap<String, Mdl> = serde_json::from_str(&std::fs::read_to_string("models.json").unwrap()).unwrap();
    let loaded_csts: HashMap<String, Cst> = serde_json::from_str(&std::fs::read_to_string("csts.json").unwrap()).unwrap();

    for (mdl_id, model) in loaded_models {
        if !system.models.contains_key(&mdl_id) {
            system.models.insert(mdl_id, model);
        }
    }
    for (cst_id, cst) in loaded_csts {
        if !system.csts.contains_key(&cst_id) {
            system.csts.insert(cst_id, cst);
        }
    }

    fn insert_sift_features(active_features: &[usize], entity: &str, system: &mut System) {
        for i in active_features {
            system.current_state.variables.insert(EntityVariableKey::new(entity, &format!("sift{i}")), Value::ConstantNumber(1.0));
        }
    }
    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![
        Value::UncertainNumber(199.99652099507048, 0.1),
        Value::UncertainNumber(-0.0006397704178381421, 0.1),
        Value::UncertainNumber(-0.002315858844667673, 0.1),
        Value::UncertainNumber(179.986083984375, 0.1)
    ]));
    system.current_state.variables.insert(EntityVariableKey::new("co1", "approximate_pos"), Value::Vec(vec![
        Value::UncertainNumber(221.99652099507048, 5.0),
        Value::UncertainNumber(21.275955974263013, 5.0),
        Value::UncertainNumber(-100.0, 5.0),
        Value::UncertainNumber(180.0, 5.0)
    ]));
    system.current_state.variables.insert(EntityVariableKey::new("h", "holding"), Value::Vec(vec![]));
    system.current_state.variables.insert(EntityVariableKey::new("co2", "approximate_pos"), Value::Vec(vec![
        Value::UncertainNumber(297.99652099507045, 5.0),
        Value::UncertainNumber(-14.468724876800815, 5.0),
        Value::UncertainNumber(-100.0, 5.0),
        Value::UncertainNumber(180.0, 5.0)
    ]));
    system.current_state.variables.insert(EntityVariableKey::new("co3", "approximate_pos"), Value::Vec(vec![
        Value::UncertainNumber(223.99652099507048, 5.0),
        Value::UncertainNumber(-54.46872487680082, 5.0),
        Value::UncertainNumber(-100.0, 5.0),
        Value::UncertainNumber(180.0, 5.0)
    ]));
    insert_sift_features(&[4, 5, 6, 7, 23], "co3", system);
    insert_sift_features(&[4, 24], "co1", system);
    insert_sift_features(&[1, 2, 3], "co2", system);


    system.goals = vec![
        /*vec![
            Fact::new(MkVal {
                entity_id: EntityPatternValue::EntityId("h".to_string()),
                var_name: "holding".to_string(),
                value: PatternItem::Value(Value::Vec(vec![
                    Value::EntityId("co1".to_string())
                ])),
                assumption: false,
            }, TimePatternRange::wildcard()),
        ],*/
        vec![
            Fact::new(MkVal {
                //entity_id: EntityPatternValue::Binding("co_o".to_string()),
                entity_id: EntityPatternValue::EntityId("co1".to_string()),
                var_name: "approximate_pos".to_string(),
                value: PatternItem::Value(Value::Vec(vec![
                    Value::UncertainNumber(228.00441002220657, 10.0),
                    Value::UncertainNumber(-49.99883096646674, 10.0),
                    Value::UncertainNumber(-100.0, 10.0),
                    Value::UncertainNumber(180.0, 10.0)
                ])),
                assumption: false,
            }, TimePatternRange::wildcard()),
            /*Fact::new(MkVal {
                entity_id: EntityPatternValue::Binding("co_o".to_string()),
                var_name: "approximate_pos".to_string(),
                value: PatternItem::Value(Value::Vec(vec![
                    Value::UncertainNumber(228.00441002220657, 10.0),
                    Value::UncertainNumber(-49.99883096646674, 10.0),
                    Value::UncertainNumber(-100.0, 10.0),
                    Value::UncertainNumber(180.0, 10.0)
                ])),
                assumption: false,
            }, TimePatternRange::wildcard()),*/
        ],
    ];
}