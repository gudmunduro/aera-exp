use std::collections::HashMap;
use std::vec;
use crate::types::cst::{Cst, ICst};
use crate::types::{Command, EntityDeclaration, EntityPatternValue, EntityVariableKey, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::functions::Function;
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::PatternItem;
use crate::types::runtime::{RuntimeCommand, System};
use crate::types::value::Value;

pub fn setup_scenario_2(system: &mut System) {
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
        },
    );

    // Move while holding the cube moves the cube

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
                        value: PatternItem::Vec(vec![PatternItem::Binding("px".to_string()), PatternItem::Binding("py".to_string()), PatternItem::Binding("pz".to_string()), PatternItem::Binding("pw".to_string())]),
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
        "M_move_cube_req".to_string(),
        Mdl {
            model_id: "M_move_cube_req".to_string(),
            left: Fact::new(
                MdlLeftValue::ICst(ICst {
                    cst_id: "S_holding".to_string(),
                    params: vec![
                        PatternItem::Binding("co".to_string()),
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
                    "M_move_cube".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("dpx".to_string()),
                        PatternItem::Binding("dpy".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("pz".to_string()),
                        PatternItem::Binding("pw".to_string()),
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
        "M_move_cube".to_string(),
        Mdl {
            model_id: "M_move_cube".to_string(),
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
                    entity_id: EntityPatternValue::Binding("co".to_string()),
                    var_name: "approximate_pos".to_string(),
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
        },
    );

    // Expected to be at 240, 0, 0, 180 before doing babble commands

    // Move to and push the blue cube
    system.babble_command.push(RuntimeCommand {
        name: "move".to_string(),
        entity_id: "h".to_string(),
        params: vec![
            Value::Vec(vec![
                Value::Number(20.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(0.0),
            ])
        ],
    });
    // Doing push here pushes the blue cube (co1) forward
    system.babble_command.push(RuntimeCommand {
        name: "push".to_string(),
        entity_id: "h".to_string(),
        params: vec![],
    });

    // Move to and push the red cube
    system.babble_command.push(RuntimeCommand {
        name: "move".to_string(),
        entity_id: "h".to_string(),
        params: vec![
            Value::Vec(vec![
                Value::Number(0.0),
                Value::Number(80.0),
                Value::Number(0.0),
                Value::Number(0.0),
            ])
        ],
    });
    system.babble_command.push(RuntimeCommand {
        name: "push".to_string(),
        entity_id: "h".to_string(),
        params: vec![],
    });


    // Pick up and move the blue cube
    system.babble_command.push(RuntimeCommand {
        name: "move".to_string(),
        entity_id: "h".to_string(),
        params: vec![
            Value::Vec(vec![
                Value::Number(50.0),
                Value::Number(-80.0),
                Value::Number(0.0),
                Value::Number(0.0),
            ])
        ],
    });
    system.babble_command.push(RuntimeCommand {
        name: "grab".to_string(),
        entity_id: "h".to_string(),
        params: vec![],
    });
    system.babble_command.push(RuntimeCommand {
        name: "move".to_string(),
        entity_id: "h".to_string(),
        params: vec![
            Value::Vec(vec![
                Value::Number(-20.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(0.0),
            ])
        ],
    });
    system.babble_command.push(RuntimeCommand {
        name: "release".to_string(),
        entity_id: "h".to_string(),
        params: vec![],
    });


    // Pick up and move the red cube
    system.babble_command.push(RuntimeCommand {
        name: "move".to_string(),
        entity_id: "h".to_string(),
        params: vec![
            Value::Vec(vec![
                Value::Number(20.0),
                Value::Number(80.0),
                Value::Number(0.0),
                Value::Number(0.0),
            ])
        ],
    });
    system.babble_command.push(RuntimeCommand {
        name: "grab".to_string(),
        entity_id: "h".to_string(),
        params: vec![],
    });
    system.babble_command.push(RuntimeCommand {
        name: "move".to_string(),
        entity_id: "h".to_string(),
        params: vec![
            Value::Vec(vec![
                Value::Number(-20.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(0.0),
            ])
        ],
    });
    system.babble_command.push(RuntimeCommand {
        name: "release".to_string(),
        entity_id: "h".to_string(),
        params: vec![],
    });

    system.goals = vec![
        vec![
            //
            // Here it should push the green cube (as that is faster to move it only by ~20)
            Fact::new(MkVal {
                entity_id: EntityPatternValue::EntityId("co3".to_string()),
                var_name: "approximate_pos".to_string(),
                value: PatternItem::Value(Value::Vec(vec![
                    Value::UncertainNumber(240.0, 10.0),
                    Value::UncertainNumber(-70.0, 10.0),
                    Value::UncertainNumber(-100.0, 10.0),
                    Value::UncertainNumber(180.0, 10.0)
                ])),
                assumption: false,
            }, TimePatternRange::wildcard()),
        ],
        vec![
            // However here we want to move it by so much that it should now pick it up to move it
            Fact::new(MkVal {
                entity_id: EntityPatternValue::EntityId("co3".to_string()),
                var_name: "approximate_pos".to_string(),
                value: PatternItem::Value(Value::Vec(vec![
                    Value::UncertainNumber(180.0, 10.0),
                    Value::UncertainNumber(0.0, 10.0),
                    Value::UncertainNumber(-100.0, 10.0),
                    Value::UncertainNumber(180.0, 10.0)
                ])),
                assumption: false,
            }, TimePatternRange::wildcard()),
        ]
    ];
}