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
#[netsblox_extension_block(name = "logHelloName", category = "Hello World", spec = "Log Hello %s", target = netsblox_extension_util::TargetObject::Both)]
pub fn hello_name(name: &str) {
    console::log_1(&format!("Hello {}!", name).to_owned().into());
}

#[netsblox_extension_label_part]
const LABEL_PART_TIMES: LabelPart = LabelPart {
    spec: "times",
    text: None,
    numeric: true,
    menu: None,
    readonly: false,
};

#[wasm_bindgen]
#[netsblox_extension_block(name = "repeatString", category = "operators", spec = "Repeat %s for %times times", target = netsblox_extension_util::TargetObject::Both)]
pub fn repeat_text(text: &str, times: f64) -> String {
    text.repeat(times as usize)
}

#[netsblox_extension_label_part]
const LABEL_PART_NUM: LabelPart = LabelPart {
    spec: "num",
    text: None,
    numeric: true,
    menu: None,
    readonly: false,
};

#[netsblox_extension_label_part]
const LABEL_PART_MENU: LabelPart = LabelPart {
    spec: "picky",
    text: None,
    numeric: true,
    menu: Some(&[
        Menu::Entry { label: "hello", value: "world" },
        Menu::Entry { label: "another", value: "option" },
        Menu::Submenu { label: "nesting", content: &[
            Menu::Submenu { label: "deeper 1", content: &[
                Menu::Entry { label: "deep 1", value: "deep val 1" },
            ] },
            Menu::Submenu { label: "deeper 2", content: &[
                Menu::Entry { label: "deep 2", value: "deep val 2" },
            ] },
        ] },
        Menu::Submenu { label: "more stuff", content: &[
            Menu::Entry { label: "thing", value: "some stuff" },
            Menu::Entry { label: "last one", value: "done" },
        ] },
    ]),
    readonly: true,
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
#[netsblox_extension_block(name = "explode", category = "control", spec = "explode", type_override = netsblox_extension_util::BlockType::Terminator, pad_top = true, pad_bottom = true)]
pub fn explode() {
    panic!()
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "addAll", category = "operators", spec = "add numbers %mult%num")]
pub fn add_all(vals: Vec<JsValue>) -> f64 {
    let mut res = 0.0;
    for val in vals {
        res += val.as_f64().unwrap_or_default();
    }
    res
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "explicitCommand", category = "control", spec = "explicit command")]
pub fn explicit_command() -> () {
    ()
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "fallibleCommand", category = "control", spec = "fallible command")]
pub fn fallible_command() -> Result<(), f64> {
    Ok(())
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "fallibleReporter", category = "control", spec = "fallible reporter", pad_top = true)]
pub fn fallible_reporter() -> Result<f64, f64> {
    Ok(12.5)
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "falliblePredicate", category = "control", spec = "fallible predicate", pad_bottom = true)]
pub fn fallible_predicate() -> Result<bool, f64> {
    Ok(true)
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "pickyboi", category = "control", spec = "picky boi %picky")]
pub fn picky_boi(v: &JsValue) -> JsValue {
    v.clone()
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "defaultAdder", category = "operators", spec = "add %n + %n", defaults = "['7', '-4']")]
pub fn default_adder(a: f64, b: f64) -> f64 {
    a + b
}
