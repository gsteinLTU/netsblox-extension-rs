use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ExtensionInfo {
    pub name: &'static str,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CustomBlock {
    pub name: &'static str,
    pub block_type: BlockType, 
    pub category: Category, 
    pub spec: &'static str, 
    pub defaults: Vec<&'static str>, 
    pub impl_fn: &'static str,
    pub target: TargetObject,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub enum TargetObject {
    SpriteMorph, StageMorph, #[default] Both
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub enum BlockType {
    #[default] #[serde(rename = "command")]
    Command, 
    #[serde(rename = "reporter")]
    Reporter, 
    #[serde(rename = "predicate")]
    Predicate
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct InputSlotMorphOptions {
    pub text: Option<&'static str>,

}

#[derive(Debug,Clone, Copy, Default, Serialize)]
pub struct LabelPart {
    pub spec: &'static str, 
    pub slot_type: InputSlotMorphOptions
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CustomCategory {
    pub name: &'static str, 
    pub color: (f64, f64, f64)
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub enum Category {
    #[default]
    #[serde(rename = "motion")]
    Motion,
    #[serde(rename = "looks")]
    Looks,
    #[serde(rename = "sound")]
    Sound,
    #[serde(rename = "pen")]
    Pen,
    #[serde(rename = "network")]
    Network,
    #[serde(rename = "control")]
    Control,
    #[serde(rename = "sensing")]
    Sensing,
    #[serde(rename = "operators")]
    Operators,
    #[serde(rename = "variables")]
    Variables,
    #[serde(rename = "custom")]
    Custom,
    CustomCategory(CustomCategory)
} 
