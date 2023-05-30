#![allow(dead_code)]

use netsblox_extension_macro::*;
use netsblox_extension_util::*;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console;

#[netsblox_extension_category]
const HELLO_WORLD_CATEGORY: CustomCategory = CustomCategory {
    name: "Hello World",
    color: (100.0, 149.0, 237.0),
};

#[netsblox_extension_info]
const INFO: ExtensionInfo = ExtensionInfo { 
    name: "ExampleExtension" 
};

#[netsblox_extension_label_part]
const LABEL_PART_NAME: LabelPart = LabelPart {
    spec: "%name",
    slot_type: InputSlotMorphOptions { text: Some("name") },
};

#[wasm_bindgen(start)]
pub fn main() {
    console::log_1(&"Hello from Rust!".to_owned().into());
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "logHelloWorld", category = "Hello World", spec = "Log Hello World!", target = netsblox_extension_util::TargetObject::Both)]
pub fn hello_world() {
    console::log_1(&"Hello World!".to_owned().into());
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "logHelloName", category = "Hello World", spec = "Log Hello %name", target = netsblox_extension_util::TargetObject::Both)]
pub fn hello_name(name: &str) {
    console::log_1(&format!("Hello {}!", name).to_owned().into());
}


#[netsblox_extension_label_part]
const LABEL_PART_TEXT: LabelPart = LabelPart {
    spec: "%text",
    slot_type: InputSlotMorphOptions { text: Some("text") },
};

#[netsblox_extension_label_part]
const LABEL_PART_TIMES: LabelPart = LabelPart {
    spec: "%times",
    slot_type: InputSlotMorphOptions { text: Some("times") },
};

#[wasm_bindgen]
#[netsblox_extension_block(name = "repeatString", category = "operators", spec = "Repeat %text for %times times", target = netsblox_extension_util::TargetObject::Both)]
pub fn repeat_text(text: &str, times: f64) -> String {
    text.repeat(times as usize)
}

#[netsblox_extension_label_part]
const LABEL_PART_NUM: LabelPart = LabelPart {
    spec: "%num",
    slot_type: InputSlotMorphOptions { text: Some("num") },
};

#[wasm_bindgen]
#[netsblox_extension_block(name = "isEven", category = "operators", spec = "is %num even?", target = netsblox_extension_util::TargetObject::Both)]
pub fn is_even(num: f64) -> bool {
    num as usize % 2 == 0
}
