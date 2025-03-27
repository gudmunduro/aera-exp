use std::vec;
use crate::types::cst::{Cst, ICst};
use crate::types::{Command, EntityDeclaration, EntityPatternValue, EntityVariableKey, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::functions::Function;
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::PatternItem;
use crate::types::runtime::System;
use crate::types::value::Value;

pub fn setup_robot_advanced_seed(system: &mut System) {
    system.create_entity("h", "hand");
    system.create_entity("c", "camera");
    system.create_entity("co1", "cam_obj");
    system.create_entity("co2", "cam_obj");
    system.create_entity("co3", "cam_obj");

    // Hand movement

    system.csts.insert(
        "S_move_h".to_string(),
        Cst {
            cst_id: "S_move_h".to_string(),
            facts: vec![Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("px".to_string()), PatternItem::Binding("py".to_string()), PatternItem::Value(Value::Number(0.0)), PatternItem::Binding("pw".to_string())]),

                    assumption: false,
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            }],
            entities: vec![EntityDeclaration::new("h", "hand")],
        },
    );

    system.models.insert(
        "mdl_move_h_req".to_string(),
        Mdl {
            model_id: "mdl_move_h_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S_move_h".to_string(),
                    params: vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("pw".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl::new(
                    "mdl_move_h".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("dpx".to_string()),
                        PatternItem::Binding("dpy".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("pw".to_string()),
                    ],
                )),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "mdl_move_h".to_string(),
        Mdl {
            model_id: "mdl_move_h".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command {
                    name: "move".to_string(),
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![PatternItem::Vec(vec![
                        PatternItem::Binding("dpx".to_string()),
                        PatternItem::Binding("dpy".to_string()),
                        PatternItem::Value(Value::Number(0.0)),
                        PatternItem::Value(Value::Number(0.0)),
                    ])],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("npx".to_string()), PatternItem::Binding("npy".to_string()), PatternItem::Value(Value::Number(0.0)), PatternItem::Binding("pw".to_string())]),
                    assumption: false,
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
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
            )].into(),
            confidence: 1.0,
        },
    );

    system.csts.insert(
        "S_move_v".to_string(),
        Cst {
            cst_id: "S_move_v".to_string(),
            facts: vec![Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("px".to_string()), PatternItem::Binding("py".to_string()), PatternItem::Binding("pz".to_string()), PatternItem::Binding("pw".to_string())]),

                    assumption: false,
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            }],
            entities: vec![EntityDeclaration::new("h", "hand")],
        },
    );

    system.models.insert(
        "mdl_move_v_req".to_string(),
        Mdl {
            model_id: "mdl_move_v_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S_move_v".to_string(),
                    params: vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("pz".to_string()),
                        PatternItem::Binding("pw".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl::new(
                    "mdl_move_v".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("dpz".to_string()),
                        PatternItem::Binding("pz".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("pw".to_string()),
                    ],
                )),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "mdl_move_v".to_string(),
        Mdl {
            model_id: "mdl_move_v".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command {
                    name: "move".to_string(),
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![PatternItem::Vec(vec![
                        PatternItem::Value(Value::Number(0.0)),
                        PatternItem::Value(Value::Number(0.0)),
                        PatternItem::Binding("dpz".to_string()),
                        PatternItem::Value(Value::Number(0.0)),
                    ])],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("px".to_string()), PatternItem::Binding("py".to_string()), PatternItem::Binding("npz".to_string()), PatternItem::Binding("pw".to_string())]),
                    assumption: false,
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [(
                "npz".to_string(),
                Function::Add(
                    Box::new(Function::Value(PatternItem::Binding("pz".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("dpz".to_string()))),
                ),
            )]
                .into(),
            backward_computed: [(
                "dpz".to_string(),
                Function::Sub(
                    Box::new(Function::Value(PatternItem::Binding("npz".to_string()))),
                    Box::new(Function::Value(PatternItem::Binding("pz".to_string()))),
                ),
            )].into(),
            confidence: 1.0,
        },
    );

    // Grab cube

    system.csts.insert(
        "S1".to_string(),
        Cst {
            cst_id: "S1".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("co".to_string()),
                        var_name: "obj_type".to_string(),
                        value: PatternItem::Value(Value::Number(0.0)),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("co".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Value(Value::Vec(vec![Value::Number(145.0), Value::Number(173.0)])),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::EntityId("h".to_string()),
                        var_name: "holding".to_string(),
                        value: PatternItem::Vec(vec![]),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::EntityId("h".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Vec(vec![PatternItem::Binding("px".to_string()), PatternItem::Binding("py".to_string()), PatternItem::Value(Value::Number(0.0)), PatternItem::Binding("pw".to_string())]),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
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
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S1".to_string(),
                    params: vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("pw".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl::new(
                    "M_grab".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("co".to_string()),
                    ],
                )),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: Default::default(),
            backward_computed: Default::default(),
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
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("co".to_string())]),
                    assumption: false,
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
                        entity_id: EntityPatternValue::Binding("h".to_string()),
                        var_name: "holding".to_string(),
                        value: PatternItem::Vec(vec![PatternItem::Binding("co".to_string())]),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("co".to_string()),
                        var_name: "color".to_string(),
                        value: PatternItem::Binding("col".to_string()),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("col".to_string()),
                        var_name: "approximate_pos".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
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
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S_holding".to_string(),
                    params: vec![
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("col".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl::new(
                    "M_release".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                    ],
                )),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: Default::default(),
            backward_computed: Default::default(),
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
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Value(Value::Vec(vec![])),
                    assumption: false,
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [].into(),
            backward_computed: [].into(),
            confidence: 1.0,
        },
    );

    // Move while holding the cube moves the cube

    /*system.models.insert(
        "M_move_cube_req".to_string(),
        Mdl {
            model_id: "M_move_cube_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S_holding".to_string(),
                    params: vec![
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("col".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl::new(
                    "M_move_cube".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("dp".to_string()),
                        PatternItem::Binding("p".to_string()),
                        PatternItem::Binding("col".to_string()),
                    ],
                )),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: Default::default(),
            backward_computed: Default::default(),
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
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![PatternItem::Binding("dp".to_string())],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("ent".to_string()),
                    var_name: "approximate_pos".to_string(),
                    value: PatternItem::Binding("np".to_string()),
                    assumption: true,
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [
                (
                    "np".to_string(),
                    Function::Add(
                        Box::new(Function::Value(PatternItem::Binding("p".to_string()))),
                        Box::new(Function::Value(PatternItem::Binding("dp".to_string()))),
                    )
                ),
                (
                    "ent".to_string(),
                    Function::ConvertToEntityId(Box::new(Function::Value(PatternItem::Binding("col".to_string()))))
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
        },
    );*/


    // Moving changes cam position of cubes

    system.csts.insert(
        "S_cube_pos".to_string(),
        Cst {
            cst_id: "S_cube_pos".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("co".to_string()),
                        var_name: "obj_type".to_string(),
                        value: PatternItem::Value(Value::Number(0.0)),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("co".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Vec(vec![PatternItem::Binding("px".to_string()), PatternItem::Binding("py".to_string())]),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
            ],
            entities: vec![
                EntityDeclaration::new("h", "hand"),
                EntityDeclaration::new("co", "cam_obj"),
            ],
        },
    );

    system.models.insert(
        "M_cube_cam_pos_req".to_string(),
        Mdl {
            model_id: "M_cube_cam_pos_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S_cube_pos".to_string(),
                    params: vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl::new(
                    "M_cube_cam_pos".to_string(),
                    vec![
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("dx".to_string()),
                        PatternItem::Binding("dy".to_string()),
                        PatternItem::Binding("px".to_string()),
                        PatternItem::Binding("py".to_string()),
                        PatternItem::Binding("co".to_string()),
                    ],
                )),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
        },
    );

    system.models.insert(
        "M_cube_cam_pos".to_string(),
        Mdl {
            model_id: "M_cube_cam_pos".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command {
                    name: "move".to_string(),
                    entity_id: EntityPatternValue::Binding("h".to_string()),
                    params: vec![PatternItem::Vec(vec![PatternItem::Binding("dx".to_string()), PatternItem::Binding("dy".to_string()), PatternItem::Value(Value::Number(0.0)), PatternItem::Value(Value::Number(0.0))])],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("co".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Vec(vec![PatternItem::Binding("npx".to_string()), PatternItem::Binding("npy".to_string())]),
                    assumption: true,
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [
                (
                    "npx".to_string(),
                    Function::Add(
                        Box::new(Function::Value(PatternItem::Binding("px".to_string()))),
                        Box::new(Function::Mul(
                            Box::new(Function::Value(PatternItem::Binding("dy".to_string()))),
                            Box::new(Function::Value(PatternItem::Value(Value::Number(1.175)))),
                        )),
                    ),
                ),
                (
                    "npy".to_string(),
                    Function::Add(
                        Box::new(Function::Value(PatternItem::Binding("py".to_string()))),
                        Box::new(Function::Value(PatternItem::Binding("dx".to_string()))),
                    )
                ),
            ].into(),
            backward_computed: [
                (
                    "dx".to_string(),
                    Function::Sub(
                        Box::new(Function::Value(PatternItem::Binding("npy".to_string()))),
                        Box::new(Function::Value(PatternItem::Binding("py".to_string()))),
                    ),
                ),
                (
                    "dy".to_string(),
                    Function::Div(
                        Box::new(Function::Sub(
                            Box::new(Function::Value(PatternItem::Binding("npx".to_string()))),
                            Box::new(Function::Value(PatternItem::Binding("px".to_string()))),
                        )),
                        Box::new(Function::Value(PatternItem::Value(Value::Number(1.175))))
                    ),
                ),
            ].into(),
            confidence: 1.0,
        },
    );


    // Assumption model for the absolute position of the cube

    system.csts.insert(
        "S_holding_obj_pos".to_string(),
        Cst {
            cst_id: "S_holding_obj_pos".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("h".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("h".to_string()),
                        var_name: "holding".to_string(),
                        value: PatternItem::Vec(vec![PatternItem::Binding("co".to_string())]),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("co".to_string()),
                        var_name: "color".to_string(),
                        value: PatternItem::Binding("col".to_string()),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
            ],
            entities: vec![
                EntityDeclaration::new("co", "cam_obj"),
                EntityDeclaration::new("h", "hand"),
            ],
        },
    );

    system.models.insert(
        "M_cube_pos_alias".to_string(),
        Mdl {
            model_id: "M_cube_pos_alias".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S_holding_obj_pos".to_string(),
                    params: vec![
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("p".to_string()),
                        PatternItem::Binding("col".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("ent".to_string()),
                    var_name: "approximate_pos".to_string(),
                    value: PatternItem::Binding("p".to_string()),
                    assumption: true,
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: [
                (
                    "ent".to_string(),
                    Function::ConvertToEntityId(Box::new(Function::Value(PatternItem::Binding("col".to_string()))))
                ),
            ].into(),
            backward_computed: [
                (
                    "col".to_string(),
                    Function::ConvertToNumber(Box::new(Function::Value(PatternItem::Binding("ent".to_string()))))
                ),
            ].into(),
            confidence: 1.0,
        },
    );

    // When I was at X, I saw a blue cube

    /*system.csts.insert(
        "S_blue_cube_memory".to_string(),
        Cst {
            cst_id: "S_blue_cube_memory".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("h".to_string()),
                        var_name: "position".to_string(),
                        value: PatternItem::Value(Value::Vec(vec![Value::Number(260.0), Value::Number(-40.0), Value::Number(0.0), Value::Number(180.0)])),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: EntityPatternValue::Binding("h".to_string()),
                        var_name: "holding".to_string(),
                        value: PatternItem::Vec(vec![]),
                        assumption: false,
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
            ],
            entities: vec![
                EntityDeclaration::new("co", "cam_obj"),
                EntityDeclaration::new("h", "hand"),
            ],
        },
    );

    system.models.insert(
        "M_blue_cube_memory".to_string(),
        Mdl {
            model_id: "M_blue_cube_memory".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "S_blue_cube_memory".to_string(),
                    params: vec![
                        PatternItem::Binding("co".to_string()),
                        PatternItem::Binding("h".to_string()),
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: EntityPatternValue::Binding("co".to_string()),
                    var_name: "color".to_string(),
                    value: PatternItem::Value(Value::Number(1.0)),
                    assumption: false,
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            forward_computed: Default::default(),
            backward_computed: Default::default(),
            confidence: 1.0,
        },
    );*/

    system.current_state.variables.insert(EntityVariableKey::new("h", "position"), Value::Vec(vec![Value::Number(0.0), Value::Number(0.0), Value::Number(0.0), Value::Number(0.0)]));
}