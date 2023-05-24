#![allow(dead_code)]

use netsblox_extension_macro::*;
use netsblox_extension_util::*;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console;

#[netsblox_extension_info]
const INFO: ExtensionInfo = ExtensionInfo { 
    name: "ExampleExtension" 
};

#[netsblox_extension_block]
const LOG_HELLO_WORLD: CustomBlock = CustomBlock { 
    name: "logHelloWorld", 
    block_type: BlockType::Command, 
    category: Category::Control, 
    spec: "Log Hello World!", 
    defaults: vec![], 
    impl_fn: "hello_world",
    target: netsblox_extension_util::TargetObject::Both
};

#[netsblox_extension_block]
const LOG_HELLO_NAME: CustomBlock = CustomBlock { 
    name: "logHelloName", 
    block_type: BlockType::Command, 
    category: Category::Control, 
    spec: "Log Hello %name", 
    defaults: vec![], 
    impl_fn: "hello_name",
    target: netsblox_extension_util::TargetObject::Both
};

#[netsblox_extension_label_part]
const LABEL_PART_NAME: LabelPart = LabelPart {
    spec: "%name",
    slot_type: InputSlotMorphOptions { text: Some("name") },
};

#[wasm_bindgen]
pub fn hello_world() {
    console::log_1(&"Hello World!".to_owned().into());
}

#[wasm_bindgen]
pub fn hello_name(name: &str) {
    console::log_1(&format!("Hello {}!", name).to_owned().into());
}