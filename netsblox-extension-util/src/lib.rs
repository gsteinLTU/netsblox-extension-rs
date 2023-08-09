use proc_macro2::{TokenTree};
use serde::Serialize;
use std::{fs::File, error::Error, io::{Read, Write}, vec, collections::{HashMap, HashSet}, path::Path};
use regex::Regex;
use simple_error::bail;
use syn::{Item, PathSegment, ItemConst, Expr, Member, Lit, ItemFn, Attribute, Meta};

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
                        // Allows for overriding block types if desired, or to make hat blocks possible
                        if let TokenTree::Ident(id) = &arg.last().unwrap() {
                            block_type_override = true;
                            match id.to_string().as_str() {
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
                    _ => {}
                }
            }
        }   
    }

    // Get information from function signature
    instance.impl_fn = Box::leak(item.sig.ident.to_string().into_boxed_str());

    if !block_type_override {
        match &item.sig.output {
            syn::ReturnType::Default => instance.block_type = BlockType::Command,
            syn::ReturnType::Type(_, b) => {
                match b.as_ref() {
                    syn::Type::Path(p) => {
                        if &p.path.segments.first().unwrap().ident.to_string() == "bool" {
                            instance.block_type = BlockType::Predicate
                        } else {
                            instance.block_type = BlockType::Reporter
                        }
                    },
                    _ => instance.block_type = BlockType::Reporter
                }
            },
        }
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
    let mut custom_blocks: HashMap<String, CustomBlock> = HashMap::new();
    let mut label_parts: HashMap<String, LabelPart> = HashMap::new();
    let mut custom_categories: HashMap<String, CustomCategory> = HashMap::new();
    let mut menu_items: HashMap<String, String> = HashMap::new();
    let mut settings: Vec<ExtensionSetting> = vec![];
    let mut fn_names: HashSet<String> = HashSet::new();

    // Parse all items
    for item in ast.items {
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
                    "netsblox_extension_label_part" => {
                        let label_part = recreate_netsblox_extension_label_part(&c);
                        warn!("Found label part block {:?}", label_part);
                        label_parts.insert(label_part.spec.to_string(), label_part);
                    },
                    "netsblox_extension_category" => {
                        let category = recreate_netsblox_extension_custom_category(&c);
                        warn!("Found custom category {:?}", category);
                        custom_categories.insert(category.name.to_string(), category);
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
                    "netsblox_extension_block" => {
                        let block = recreate_netsblox_extension_block(&f, attr);

                        if !block.name.is_empty() {
                            warn!("Found custom block {:?}", block);
                            custom_blocks.insert(block.name.to_string(), block.clone());
                            fn_names.insert(block.impl_fn.to_string());
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
                                menu_items.insert(menu_text, fn_name.to_string());
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
            menu_string += format!("\t\t\t\t'{}': window.{}_fns.{},\n", label, extension_name_no_spaces.as_str(), fn_name).as_str();
        }

        content = content.replace("$MENU", &menu_string);

        let mut settings_string = "".to_string();

        for setting in settings {
            settings_string += format!("\t\t\t\tExtension.ExtensionSetting.createFromLocalStorage('{}', '{}', {}, '{}', '{}', {}),\n", setting.name, setting.id, setting.default_value, setting.on_hint, setting.off_hint, setting.hidden).as_str();
        }

        content = content.replace("$SETTINGS", &settings_string);
        
        let mut categories_string = "".to_string();

        for cat in custom_categories.values() {
            categories_string += format!("\t\t\t\tnew Extension.Category('{}', new Color({}, {}, {})),\n", cat.name, cat.color.0, cat.color.1, cat.color.2).as_str();
        }

        content = content.replace("$CATEGORIES", &categories_string);

        let mut palette_string = "".to_string();

        let mut categories_map: HashMap<String, Vec<String>> = HashMap::new();

        for block in custom_blocks.values() {
            let block_cat = serde_json::to_string(&block.category)?.strip_prefix('\"').unwrap().strip_suffix('\"').unwrap().to_string();

            if !categories_map.contains_key(&block_cat) {
                categories_map.insert(block_cat.clone(), vec![]);
            }

            categories_map.get_mut(&block_cat).unwrap().push(block.name.to_string());
        }
        

        for category in categories_map.keys() {
            palette_string += "\t\t\t\tnew Extension.PaletteCategory(\n";
            palette_string += format!("\t\t\t\t\t'{}',\n", category).as_str();
            palette_string += "\t\t\t\t\t[\n";
            for block_name in categories_map.get(category).unwrap() {
                if custom_blocks.get(block_name).unwrap().target == TargetObject::SpriteMorph || custom_blocks.get(block_name).unwrap().target == TargetObject::Both {
                    palette_string += format!("\t\t\t\t\t\tnew Extension.Palette.Block('{}'),\n", block_name).as_str();
                }
            }
            palette_string += "\t\t\t\t\t],\n";
            palette_string += "\t\t\t\t\tSpriteMorph\n";
            palette_string += "\t\t\t\t),\n";
            
            palette_string += "\t\t\t\tnew Extension.PaletteCategory(\n";
            palette_string += format!("\t\t\t\t\t'{}',\n", category).as_str();
            palette_string += "\t\t\t\t\t[\n";
            for block_name in categories_map.get(category).unwrap() {
                if custom_blocks.get(block_name).unwrap().target == TargetObject::StageMorph || custom_blocks.get(block_name).unwrap().target == TargetObject::Both {
                    palette_string += format!("\t\t\t\t\t\tnew Extension.Palette.Block('{}'),\n", block_name).as_str();
                }
            }
            palette_string += "\t\t\t\t\t],\n";
            palette_string += "\t\t\t\t\tStageMorph\n";
            palette_string += "\t\t\t\t),\n";
        }

        content = content.replace("$PALETTE", &palette_string);


        let mut blocks_str = "".to_string();

        let label_parts_regex = Regex::new("%(\\w+)")?;

        for block in custom_blocks.values() {
            blocks_str += "\t\t\t\tnew Extension.Block(\n";
            blocks_str += format!("\t\t\t\t\t'{}',\n", block.name).as_str();
            blocks_str += format!("\t\t\t\t\t'{}',\n", serde_json::to_string(&block.block_type)?.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap()).as_str();
            blocks_str += format!("\t\t\t\t\t'{}',\n", serde_json::to_string(&block.category)?.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap()).as_str();
            blocks_str += format!("\t\t\t\t\t'{}',\n", block.spec).as_str();
            blocks_str += format!("\t\t\t\t\t[],\n").as_str();

            let label_parts_str = label_parts_regex.captures_iter(block.spec).map(|c| {
                c.iter().last().unwrap().unwrap().as_str()
            }).collect::<Vec<&str>>().join(", ");

            blocks_str += format!("\t\t\t\t\tfunction ({}) {{ return {}_fns.{}({}); }}\n", label_parts_str, extension_name_no_spaces.as_str(), block.impl_fn, label_parts_str).as_str();
            blocks_str += "\t\t\t\t).for(SpriteMorph, StageMorph),\n";

            // Add default label parts
            for label_part in Regex::new("%\\w+").unwrap().find_iter(block.spec) {
                let label_part = label_part.as_str();

                if !label_parts.contains_key(label_part) {
                    label_parts.insert(label_part.to_string(), LabelPart { 
                        spec: label_part.clone(), 
                        slot_type: InputSlotMorphOptions::default() 
                    });
                }
            }
        }

        content = content.replace("$BLOCKS", blocks_str.as_str());
        
        let mut label_parts_string = "".to_string();

        for label_part in label_parts.values() {
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

        content = content.replace("$IMPORTS_LIST", &fn_names.iter().map(|s| s.to_owned()).collect::<Vec<String>>().join(", "));
        content = content.replace("$WINDOW_IMPORTS", &fn_names.iter().map(|fn_name| format!("\t\twindow.{}_fns.{} = {};", extension_name_no_spaces.as_str(), fn_name, fn_name)).collect::<Vec<String>>().join("\n"));
        
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