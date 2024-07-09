#![allow(dead_code)]

use netsblox_extension_macro::*;
use netsblox_extension_util::*;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::console;
extern crate console_error_panic_hook;
use std::panic;

#[netsblox_extension_category]
const HELLO_WORLD_CATEGORY: CustomCategory = CustomCategory {
    name: "Hello World",
    color: (100.0, 149.0, 237.0),
};

#[netsblox_extension_info]
const INFO: ExtensionInfo = ExtensionInfo { 
    name: "Example Extension" 
};

#[netsblox_extension_setting]
const CAPS_SETTING: ExtensionSetting = ExtensionSetting {
    name: "All Caps output from Menu Item",
    id: "exampleextensionallcaps",
    default_value: false,
    on_hint: "Capitalize output",
    off_hint: "Do not capitalize output",
    hidden: false,
};

#[wasm_bindgen(start)]
pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
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
const LABEL_PART_TIMES: LabelPart = LabelPart {
    spec: "%times",
    slot_type: InputSlotMorphOptions { text: Some("times"), is_numeric: true },
};

#[wasm_bindgen]
#[netsblox_extension_block(name = "repeatString", category = "operators", spec = "Repeat %text for %times times", target = netsblox_extension_util::TargetObject::Both)]
pub fn repeat_text(text: &str, times: f64) -> String {
    text.repeat(times as usize)
}

#[netsblox_extension_label_part]
const LABEL_PART_NUM: LabelPart = LabelPart {
    spec: "%num",
    slot_type: InputSlotMorphOptions { text: Some("num"), is_numeric: true },
};

#[wasm_bindgen]
#[netsblox_extension_block(name = "isEven", category = "operators", spec = "is %num even?", target = netsblox_extension_util::TargetObject::Both)]
pub fn is_even(num: f64) -> bool {
    num as usize % 2 == 0
}


#[wasm_bindgen]
#[netsblox_extension_block(name = "receiveTestEvent", category = "control", spec = "on test event", type_override = netsblox_extension_util::BlockType::Hat, target = netsblox_extension_util::TargetObject::Both)]
pub fn receive_test_event() { }

#[wasm_bindgen]
#[netsblox_extension_menu_item("Print Hello World")]
pub fn print_hello_world() {
    if CAPS_SETTING.get() {
        console::log_1(&"Hello World".to_owned().to_uppercase().into());
    } else {
        console::log_1(&"Hello World".to_owned().into());
    }
}

#[wasm_bindgen]
#[netsblox_extension_menu_item("Print Extension Name")]
pub fn print_extension_name() {
    if CAPS_SETTING.get() {
        console::log_1(&INFO.name.to_owned().to_uppercase().into());
    } else {
        console::log_1(&INFO.name.to_owned().into());
    }
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "printProcess", category = "control", spec = "print process", target = netsblox_extension_util::TargetObject::Both, pass_proc = true)]
pub fn print_process(this: JsValue) {
    console::log_1(&this);
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "explode", category = "control", spec = "explode", type_override = netsblox_extension_util::BlockType::Terminator)]
pub fn explode() {
    panic!()
}
