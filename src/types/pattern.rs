pub type Pattern = Vec<PatternItem>;

#[derive(Clone, Debug, PartialEq)]
pub enum PatternItem {
    Any,
    Binding(String),
    Value(PatternValue),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternValue {
    String(String),
    Number(f64),
}