use proc_macro2::TokenTree;
use serde::Serialize;
use std::{fs::File, error::Error, io::{Read, Write}, vec, collections::{HashMap, HashSet}, path::Path, fmt::Write as FmtWrite};
use regex::Regex;
use simple_error::bail;
use syn::{Item, PathSegment, ItemConst, Expr, Member, Lit, ItemFn, Attribute, Meta};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ExtensionInfo {
    pub name: &'static str,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CustomBlock {
    pub name: &'static str,
    pub block_type: BlockType,
    pub category: &'static str,
    pub spec: &'static str,
    pub defaults: Vec<&'static str>,
    pub impl_fn: &'static str,
    pub target: TargetObject,
    pub pass_proc: bool,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub enum TargetObject {
    SpriteMorph, StageMorph, #[default] Both
}

#[derive(Debug, Clone, Copy, Serialize, Default, PartialEq, Eq)]
pub enum BlockType {
    #[default] #[serde(rename = "command")]
    Command,
    #[serde(rename = "command")]
    Terminator,
    #[serde(rename = "reporter")]
    Reporter,
    #[serde(rename = "predicate")]
    Predicate,
    #[serde(rename = "hat")]
    Hat
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct InputSlotMorphOptions {
    pub text: Option<&'static str>,
    pub is_numeric: bool
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

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ExtensionSetting {
    pub name: &'static str,
    pub id: &'static str,
    pub default_value: bool,
    pub on_hint: &'static str,
    pub off_hint: &'static str,
    pub hidden: bool
}

impl ExtensionSetting {
    pub fn get(&self) -> bool {
        let window = &web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        storage.get_item(self.id).unwrap().unwrap_or(if self.default_value { "true".to_string() } else { "false".to_string()} ) == "true"
    }

    pub fn set(&self, val: bool) {
        let window = web_sys::window();
        let local_storage = window.unwrap().local_storage().unwrap().unwrap();
        local_storage.set_item(self.id, if val {"true"} else {"false"}).unwrap();
    }
}

// Macro to allow build script to print output
macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

// Turn syn item into instance
fn recreate_netsblox_extension_info(item: &ItemConst) -> ExtensionInfo {
    let mut instance = ExtensionInfo::default();

    if let Expr::Struct(s) = &*item.expr {
        for field in &s.fields {
            if let Member::Named(named) = &field.member {
                let name = named.to_string();
                {
                    match name.as_str() {
                        "name" => instance.name = extract_string(&field.expr),
                        _ => warn!("Unknown field: {}", name)
                    }
                }
            }
        }
    }
    instance
}

// Turn syn item into instance
fn recreate_netsblox_extension_custom_category(item: &ItemConst) -> CustomCategory {
    let mut instance = CustomCategory::default();

    if let Expr::Struct(s) = &*item.expr {
        for field in &s.fields {
            if let Member::Named(named) = &field.member {
                let name = named.to_string();
                {
                    match name.as_str() {
                        "name" => instance.name = extract_string(&field.expr),
                        "color" => {
                            if let Expr::Tuple(t) = &field.expr {
                                let colors = &t.elems.iter().collect::<Vec<_>>();

                                if colors.len() < 3 {
                                    warn!("Invalid color for category {}", instance.name);
                                } else {
                                    instance.color.0 = extract_f64(colors[0]);
                                    instance.color.1 = extract_f64(colors[1]);
                                    instance.color.2 = extract_f64(colors[2]);
                                }
                            }
                        },
                        _ => warn!("Unknown field: {}", name)
                    }
                }
            }
        }
    }
    instance
}

// Turn syn item into instance
fn recreate_netsblox_extension_block(item: &ItemFn, attr: &Attribute) -> CustomBlock {
    let mut instance = CustomBlock::default();
    let mut block_type_override = false;

    // Parse information stored in attribute
    if let Meta::List(l) = &attr.meta {
        let t = &l.tokens.clone().into_iter().collect::<Vec<_>>();

        let args = t.split(|tt| {
            if let TokenTree::Punct(p) = tt {
                return p.as_char() == ',';
            }

            false
        }).collect::<Vec<_>>();

        for arg in args {
            if let TokenTree::Ident(i) = arg.first().unwrap() {
                let sym = i.to_string();
                match sym.as_str() {
                    "name" => {
                        if let TokenTree::Literal(lit) = &arg[2] {
                            instance.name = Box::leak(lit.to_string().replace('"', "").into_boxed_str());
                        }
                    },
                    "category" => {
                        if let TokenTree::Literal(lit) = &arg[2] {
                            instance.category = Box::leak(lit.to_string().replace('"', "").into_boxed_str());
                        }
                    },
                    "spec" => {
                        if let TokenTree::Literal(lit) = &arg[2] {
                            instance.spec = Box::leak(lit.to_string().replace('"', "").into_boxed_str());
                        }
                    },
                    "target" => {
                        // For now, defaults to targetting both until we have a use-case justifying it, the library doesn't support enough NetsBlox interaction to make sprites vs stage meaningful yet
                        //warn!("{:?}", arg)
                    },
                    "type_override" => {
                        // Allows for overriding block types if desired, or to make hat/terminal blocks possible
                        if let TokenTree::Ident(id) = &arg.last().unwrap() {
                            block_type_override = true;
                            match id.to_string().as_str() {
                                "Terminator" => { instance.block_type = BlockType::Terminator },
                                "Command" => { instance.block_type = BlockType::Command },
                                "Reporter" => { instance.block_type = BlockType::Reporter },
                                "Predicate" => { instance.block_type = BlockType::Predicate },
                                "Hat" => { instance.block_type = BlockType::Hat },
                                _ => {
                                    warn!("Unrecognized block type override type: {:?}", id);
                                    block_type_override = false;
                                }
                            }
                        }
                    },
                    "pass_proc" => {
                        if arg[2].to_string() == "true" {
                            instance.pass_proc = true;
                        }
                    }
                    x => warn!("Unknown field: {x}"),
                }
            }
        }
    }

    // Get information from function signature
    instance.impl_fn = Box::leak(item.sig.ident.to_string().into_boxed_str());

    if !block_type_override {
        instance.block_type = match &item.sig.output {
            syn::ReturnType::Default => BlockType::Command,
            syn::ReturnType::Type(_, b) => match b.as_ref() {
                syn::Type::Path(p) if &p.path.segments.first().unwrap().ident.to_string() == "bool" => BlockType::Predicate,
                _ => BlockType::Reporter
            },
        };
    }

    instance
}


// Turn syn item into instance
fn recreate_netsblox_extension_label_part(item: &ItemConst) -> LabelPart {
    let mut instance = LabelPart::default();

    if let Expr::Struct(s) = &*item.expr {
        for field in &s.fields {
            if let Member::Named(named) = &field.member {

                let name = named.to_string();
                {
                    match name.as_str() {
                        "spec" => instance.spec = extract_string(&field.expr),
                        "slot_type" => {
                            if let Expr::Struct(slot_type) = &field.expr {
                                let mut slot_type_instance = InputSlotMorphOptions::default();

                                for field in &slot_type.fields {

                                    if let Member::Named(named) = &field.member {
                                        match named.to_string().as_str() {
                                            "text" => slot_type_instance.text = Some(extract_string(&field.expr)),
                                            "is_numeric" => slot_type_instance.is_numeric = extract_bool(&field.expr),
                                            _ => warn!("Unknown input slot morph options field {}", named.to_string())
                                        }
                                    }
                                }

                                instance.slot_type = slot_type_instance;
                            }
                        },
                        _ => warn!("Unknown field: {}", name)
                    }
                }
            }
        }
    }
    instance
}

fn recreate_netsblox_extension_setting(item: &ItemConst) -> ExtensionSetting {
    let mut instance = ExtensionSetting::default();

    if let Expr::Struct(s) = &*item.expr {
        for field in &s.fields {
            if let Member::Named(named) = &field.member {
                let name = named.to_string();
                {
                    match name.as_str() {
                        "name" => instance.name = extract_string(&field.expr),
                        "id" => instance.id = extract_string(&field.expr),
                        "on_hint" => instance.on_hint = extract_string(&field.expr),
                        "off_hint" => instance.off_hint = extract_string(&field.expr),
                        "default_value" => instance.default_value = extract_bool(&field.expr),
                        "hidden" => instance.hidden = extract_bool(&field.expr),
                        _ => warn!("Unknown field: {}", name)
                    }
                }
            }
        }
    }
    instance
}

fn extract_string(expr: &syn::Expr) -> &'static str {
    if let Expr::Lit(lit) = expr {
        if let Lit::Str(val) = &lit.lit {
            let val = val.value();
            // Leaking would be bad, but this script has a short life
            return Box::leak(val.into_boxed_str());
        }
    }

    ""
}

fn extract_bool(expr: &syn::Expr) -> bool {
    if let Expr::Lit(lit) = expr {
        if let Lit::Bool(val) = &lit.lit {
            return val.value;
        }
    }

    false
}

fn extract_f64(expr: &syn::Expr) -> f64 {
    if let Expr::Lit(lit) = expr {
        if let Lit::Float(val) = &lit.lit {
            return val.base10_parse().unwrap();
        }
    }

    0.0
}

pub fn build() -> Result<(), Box<dyn Error>>  {
    // Read file
    let mut file = File::open("./src/lib.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let ast = syn::parse_file(&content)?;

    let mut extension_info: Option<ExtensionInfo> = None;
    let mut custom_blocks: Vec<(String, CustomBlock)> = vec![];
    let mut label_parts: Vec<(&str, LabelPart)> = vec![];
    let mut custom_categories: Vec<(String, CustomCategory)> = vec![];
    let mut menu_items: Vec<(String, String)> = vec![];
    let mut settings: Vec<ExtensionSetting> = vec![];
    let mut fn_names: HashSet<String> = HashSet::new();
    let mut setup_func: Option<String> = None;

    // Start with built-in label part specifiers
    let mut known_label_parts: BTreeSet<&str> = include_str!("builtin-types.txt").lines().map(|x| x.trim()).filter(|x| !x.is_empty()).collect();

    let label_parts_regex = Regex::new(r"(%mult)?%(\w+)")?;

    // Parse label parts
    for item in &ast.items {
        // Definitions will be consts
        if let Item::Const(c) = item {
            // Check for attributes
            for attr in &c.attrs {
                let seg = attr.meta.path().segments.first().unwrap() as &PathSegment;
                let ident = seg.ident.to_string();

                match ident.as_str() {
                    "netsblox_extension_label_part" => {
                        let label_part = recreate_netsblox_extension_label_part(&c);
                        warn!("Found label part block {:?}", label_part);
                        label_parts.push((label_part.spec, label_part));
                        known_label_parts.insert(label_part.spec);
                    },
                    _ => {}
                };
            }
        }
    }

    warn!("Known label parts: {:?}", known_label_parts);

    // Parse all other items
    for item in &ast.items {
        // Definitions will be consts
        if let Item::Const(c) = item {
            // Check for attributes
            for attr in &c.attrs {
                let seg = attr.meta.path().segments.first().unwrap() as &PathSegment;
                let ident = seg.ident.to_string();

                match ident.as_str() {
                    "netsblox_extension_info" => {
                        extension_info = Some(recreate_netsblox_extension_info(&c));
                        warn!("Found extension info {:?}", extension_info);
                    },
                    "netsblox_extension_category" => {
                        let category = recreate_netsblox_extension_custom_category(&c);
                        warn!("Found custom category {:?}", category);
                        custom_categories.push((category.name.to_string(), category));
                    },
                    "netsblox_extension_setting" => {
                        let setting: ExtensionSetting = recreate_netsblox_extension_setting(&c);
                        warn!("Found setting {}", setting.name);
                        settings.push(setting);
                    },
                    _ => {}
                };
            }
        } else if let Item::Fn(f) = item  {
            // Check for attributes
            for attr in &f.attrs {
                let seg = attr.meta.path().segments.first().unwrap() as &PathSegment;
                let ident = seg.ident.to_string();

                match ident.as_str() {
                    "netsblox_extension_setup" => {
                        let name = f.sig.ident.to_string();
                        warn!("Found setup function {name:?}");

                        if f.sig.inputs.len() != 0 {
                            panic!("Setup function cannot have inputs!");
                        }
                        if let Some(prev_setup) = &setup_func {
                            panic!("Multiple setup functions encountered! ({prev_setup} and {name})");
                        }

                        fn_names.insert(name.clone());
                        setup_func = Some(name);
                    }
                    "netsblox_extension_block" => {
                        let block = recreate_netsblox_extension_block(&f, attr);

                        if !block.name.is_empty() {
                            warn!("Found custom block {:?}", block);
                            custom_blocks.push((block.name.to_string(), block.clone()));
                            fn_names.insert(block.impl_fn.to_string());

                            // Check if label parts used by block spec are known
                            for cap in label_parts_regex.captures_iter(block.spec) {
                                let label_part = cap.get(2).unwrap().as_str();
                                if !known_label_parts.contains(&label_part) {
                                    panic!("Unknown label part %{}!", label_part);
                                }
                            }
                        } else {
                            warn!("Invalid custom block found");
                        }
                    },
                    "netsblox_extension_menu_item" => {
                        let fn_name = Box::leak(f.sig.ident.to_string().into_boxed_str());

                        if let Meta::List(l) = &attr.meta {
                            let t = &l.tokens.clone().into_iter().collect::<Vec<_>>();

                            let args = t.split(|tt| {
                                if let TokenTree::Punct(p) = tt {
                                    return p.as_char() == ',';
                                }

                                false
                            }).collect::<Vec<_>>();

                            if let Some(arg) = args.first() {
                                let menu_text = arg.first().unwrap().to_string().replace('"', "");
                                warn!("Found menu item {} for fn {}", menu_text, fn_name);
                                menu_items.push((menu_text, fn_name.to_string()));
                                fn_names.insert(fn_name.to_string());
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    if let Some(extension_info) = extension_info {
        let mut content = include_str!("./template.js").to_string();

        content = content.replace("$EXTENSION_NAME", extension_info.name);
        let extension_name_no_spaces = extension_info.name.replace(" ", "");
        content = content.replace("$NO_SPACE_EXTENSION_NAME", extension_name_no_spaces.as_str());

        let mut menu_string = "".to_string();

        for (label, fn_name) in menu_items {
            write!(menu_string, "\t\t\t\t'{label}': window.{extension_name_no_spaces}_fns.{fn_name},\n").unwrap();
        }

        content = content.replace("$MENU", &menu_string);

        let mut settings_string = "".to_string();

        for setting in settings {
            write!(settings_string, "\t\t\t\tExtension.ExtensionSetting.createFromLocalStorage('{}', '{}', {}, '{}', '{}', {}),\n", setting.name, setting.id, setting.default_value, setting.on_hint, setting.off_hint, setting.hidden).unwrap();
        }

        content = content.replace("$SETTINGS", &settings_string);

        let mut categories_string = "".to_string();

        for (_, cat) in custom_categories {
            write!(categories_string, "\t\t\t\tnew Extension.Category('{}', new Color({}, {}, {})),\n", cat.name, cat.color.0, cat.color.1, cat.color.2).unwrap();
        }

        content = content.replace("$CATEGORIES", &categories_string);

        let mut palette_string = "".to_string();

        let mut categories_map: HashMap<String, Vec<String>> = HashMap::new();

        for (_, block) in &custom_blocks {
            let block_cat = serde_json::to_string(&block.category)?.strip_prefix('\"').unwrap().strip_suffix('\"').unwrap().to_string();

            if !categories_map.contains_key(&block_cat) {
                categories_map.insert(block_cat.clone(), vec![]);
            }

            categories_map.get_mut(&block_cat).unwrap().push(block.name.to_string());
        }


        let mut cat_names: Vec<_> = categories_map.keys().collect();
        cat_names.sort_unstable();

        for category in cat_names {
            palette_string += "\t\t\t\tnew Extension.PaletteCategory(\n";
            palette_string += format!("\t\t\t\t\t'{}',\n", category).as_str();
            palette_string += "\t\t\t\t\t[\n";
            for block_name in categories_map.get(category).unwrap() {
                let get = &custom_blocks.iter().find(|(b, _)| b == block_name).unwrap().1;
                if get.target == TargetObject::SpriteMorph || get.target == TargetObject::Both {
                    write!(palette_string, "\t\t\t\t\t\tnew Extension.Palette.Block('{}'),\n", block_name).unwrap();
                }
            }
            palette_string += "\t\t\t\t\t],\n";
            palette_string += "\t\t\t\t\tSpriteMorph\n";
            palette_string += "\t\t\t\t),\n";

            palette_string += "\t\t\t\tnew Extension.PaletteCategory(\n";
            palette_string += format!("\t\t\t\t\t'{}',\n", category).as_str();
            palette_string += "\t\t\t\t\t[\n";
            for block_name in categories_map.get(category).unwrap() {
                let get = &custom_blocks.iter().find(|(b, _)| b == block_name).unwrap().1;
                if get.target == TargetObject::StageMorph || get.target == TargetObject::Both {
                    write!(palette_string, "\t\t\t\t\t\tnew Extension.Palette.Block('{}'),\n", block_name).unwrap();
                }
            }
            palette_string += "\t\t\t\t\t],\n";
            palette_string += "\t\t\t\t\tStageMorph\n";
            palette_string += "\t\t\t\t),\n";
        }

        content = content.replace("$PALETTE", &palette_string);


        let mut blocks_str = "".to_string();

        for (_, block) in &custom_blocks {
            blocks_str += "\t\t\t\tnew Extension.Block(\n";
            blocks_str += format!("\t\t\t\t\t'{}',\n", block.name).as_str();
            blocks_str += format!("\t\t\t\t\t'{}',\n", serde_json::to_string(&block.block_type)?.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap()).as_str();
            blocks_str += format!("\t\t\t\t\t'{}',\n", serde_json::to_string(&block.category)?.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap()).as_str();
            blocks_str += format!("\t\t\t\t\t'{}',\n", block.spec).as_str();
            blocks_str += "\t\t\t\t\t[],\n";

            let label_parts_str = label_parts_regex.captures_iter(block.spec).enumerate().map(|(i, _)| format!("v{i}")).collect::<Vec<_>>().join(", ");

            let proc_token = if block.pass_proc { "this, " } else { "" };
            let terminal_token = if block.block_type == BlockType::Terminator { ".terminal()" } else { "" };

            write!(blocks_str, "\t\t\t\t\tfunction ({label_parts_str}) {{ return window.{extension_name_no_spaces}_fns.{}({proc_token}{label_parts_str}); }}\n", block.impl_fn).unwrap();
            write!(&mut blocks_str, "\t\t\t\t){terminal_token}.for(SpriteMorph, StageMorph),\n").unwrap();
        }

        content = content.replace("$BLOCKS", blocks_str.as_str());

        let mut label_parts_string = "".to_string();

        for (_, label_part) in label_parts {
            label_parts_string += "\t\t\t\tnew Extension.LabelPart(\n";
            label_parts_string += format!("\t\t\t\t\t'{}',\n", label_part.spec).as_str();
            label_parts_string += "\t\t\t\t\t() => {\n";
            label_parts_string += "\t\t\t\t\t\tconst part = new InputSlotMorph(\n";
            label_parts_string += "\t\t\t\t\t\t\tnull, // text\n";
            label_parts_string += format!("\t\t\t\t\t\t\t{}, // is numeric\n", label_part.slot_type.is_numeric).as_str();
            label_parts_string += "\t\t\t\t\t\t\tnull,\n";
            label_parts_string += "\t\t\t\t\t\t\tfalse\n";
            label_parts_string += "\t\t\t\t\t\t);\n";
            label_parts_string += "\t\t\t\t\t\treturn part;\n";
            label_parts_string += "\t\t\t\t\t}\n";
            label_parts_string += "\t\t\t\t),\n";
        }

        content = content.replace("$LABELPARTS", &label_parts_string);

        let mut fn_names = fn_names.iter().cloned().collect::<Vec<String>>();
        fn_names.sort_unstable();
        content = content.replace("$IMPORTS_LIST", &fn_names.iter().map(|s| s.to_owned()).collect::<Vec<_>>().join(", "));
        content = content.replace("$WINDOW_IMPORTS", &fn_names.iter().map(|fn_name| format!("\t\twindow.{extension_name_no_spaces}_fns.{fn_name} = {fn_name};")).collect::<Vec<_>>().join("\n"));
        content = content.replace("$SETUP_FUNC", &setup_func.map(|f| format!("\t\t{f}();")).unwrap_or_default());

        let mut package = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let p = Path::new(package.as_str());
        package = p.file_name().unwrap().to_str().unwrap().to_string();
        package = package.replace("-", "_");
        content = content.replace("$PACKAGE_NAME", package.as_str());

        let mut out_file = File::create("./index.js")?;
        out_file.write_all(content.as_bytes())?;
    } else {
        bail!("No ExtensionInfo found!");
    }

    Ok(())
}