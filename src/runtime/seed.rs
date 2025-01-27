use crate::types::cst::{Cst, ICst};
use crate::types::{Command, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{PatternItem, PatternValue};
use crate::types::runtime::RuntimeData;

pub fn setup_seed(data: &mut RuntimeData) {
    data.csts.insert(
        "cst_pos".to_string(),
        Cst {
            cst_id: "cst_pos".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: "h".to_string(),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                }
            ],
        }
    );

    data.models.insert(
        "mdl_move_req".to_string(),
        Mdl {
            model_id: "mdl_move_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "cst_pos".to_string(),
                    pattern: vec![
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "mdl_move".to_string(),
                    params: vec![
                        PatternItem::Binding("np".to_string()),
                    ]
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            confidence: 1.0,
        },
    );

    data.models.insert(
        "mdl_move".to_string(),
        Mdl {
            model_id: "mdl_move".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command{ name: "move".to_string(), params: vec![
                    PatternItem::Binding("np".to_string()),
                ] }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: "h".to_string(),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("np".to_string())
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            confidence: 1.0,
        },
    );

    data.csts.insert(
        "cst_cube".to_string(),
        Cst {
            cst_id: "cst_cube".to_string(),
            facts: vec![
                Fact {
                    pattern: MkVal {
                        entity_id: "h".to_string(),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                },
                Fact {
                    pattern: MkVal {
                        entity_id: "c".to_string(),
                        var_name: "position".to_string(),
                        value: PatternItem::Binding("p".to_string()),
                    },
                    time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
                }
            ],
        }
    );

    data.models.insert(
        "mdl_move_cube_req".to_string(),
        Mdl {
            model_id: "mdl_move_cube_req".to_string(),
            left: Fact {
                pattern: MdlLeftValue::ICst(ICst {
                    cst_id: "cst_cube".to_string(),
                    pattern: vec![
                        PatternItem::Binding("p".to_string()),
                    ],
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::IMdl(IMdl {
                    model_id: "mdl_move_cube".to_string(),
                    params: vec![
                        PatternItem::Binding("np".to_string()),
                    ]
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            confidence: 1.0,
        },
    );

    data.models.insert(
        "mdl_move_cube".to_string(),
        Mdl {
            model_id: "mdl_move_cube".to_string(),
            left: Fact {
                pattern: MdlLeftValue::Command(Command{ name: "move_cube".to_string(), params: vec![
                    PatternItem::Binding("np".to_string()),
                ] }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            right: Fact {
                pattern: MdlRightValue::MkVal(MkVal {
                    entity_id: "h".to_string(),
                    var_name: "position".to_string(),
                    value: PatternItem::Binding("np".to_string())
                }),
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any),
            },
            confidence: 1.0,
        },
    );
}
