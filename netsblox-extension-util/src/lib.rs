use serde::Serialize;
use std::{fs::File, error::Error, io::{Read, Write}, vec, collections::HashMap};
use regex::Regex;
use simple_error::bail;
use syn::{Item, PathSegment, ItemConst, Expr, Member, Lit};

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
                        "name" => instance.name = extract_string(field),
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
                        "name" => instance.name = extract_string(field),
                        "color" => {},
                        _ => warn!("Unknown field: {}", name) 
                    }
                }
            }
        }
    }
    instance
}

// Turn syn item into instance
fn recreate_netsblox_extension_block(item: &ItemConst) -> CustomBlock {
    let mut instance = CustomBlock::default();

    if let Expr::Struct(s) = &*item.expr {
        for field in &s.fields {
            if let Member::Named(named) = &field.member { 
                
                let name = named.to_string();
                {
                    match name.as_str() {
                        "name" => instance.name = extract_string(field),
                        "block_type" => {
                            if let Expr::Path(p) = &field.expr {
                                let block_type = &p.path.segments.last().unwrap().ident.to_string();
                                match block_type.as_str() {
                                    "Command" => instance.block_type = BlockType::Command,
                                    "Reporter" => instance.block_type = BlockType::Reporter,
                                    "Predicate" => instance.block_type = BlockType::Predicate,
                                    _ => warn!("Unknown block type {}", block_type)
                                }
                            }
                        },
                        "category" => instance.category = extract_string(field),
                        "spec" => instance.spec = extract_string(field),
                        "defaults" => {
                            // TODO
                        },
                        "impl_fn" => instance.impl_fn = extract_string(field),
                        "target" => {},
                        _ => warn!("Unknown field: {}", name)
                    }
                }
            }
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
                        "spec" => instance.spec = extract_string(field),
                        _ => warn!("Unknown field: {}", name)
                    }
                }
            }
        }
    }
    instance
}

fn extract_string(field: &syn::FieldValue) -> &'static str {
    if let Expr::Lit(lit) = &field.expr {
        if let Lit::Str(val) = &lit.lit {
            let val = val.value();
            // Leaking would be bad, but this script has a short life
            return Box::leak(val.into_boxed_str());
        }
    }
    
    return "";
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
    let mut fn_names: Vec<String> = vec![];

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
                    }
                    "netsblox_extension_block" => {
                        let block = recreate_netsblox_extension_block(&c);
                        warn!("Found custom block {:?}", block);
                        custom_blocks.insert(block.name.to_string(), block.clone());
                        fn_names.push(block.impl_fn.to_string());
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
                    _ => {}
                };
            }
        }
    }

    if let Some(extension_info) = extension_info {
        let mut content = include_str!("./template.js").to_string();

        content = content.replace("$EXTENSION_NAME", extension_info.name);
        content = content.replace("$MENU", "");
        content = content.replace("$SETTINGS", "");
        
        let mut categories_string = "".to_string();

        for cat in custom_categories.values() {
            categories_string += format!("\t\t\t\tnew Extension.Category('{}', new Color(195, 0, 204)),\n", cat.name).as_str();
        }

        content = content.replace("$CATEGORIES", &categories_string);

        let mut palette_string = "".to_string();

        let mut categories_map: HashMap<String, Vec<String>> = HashMap::new();

        for block in custom_blocks.values() {
            let block_cat = serde_json::to_string(&block.category)?.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap().to_string();

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

            blocks_str += format!("\t\t\t\t\tfunction ({}) {{ {}_fns.{}({}) }}\n", label_parts_str, extension_info.name, block.impl_fn, label_parts_str).as_str();
            blocks_str += "\t\t\t\t).for(SpriteMorph, StageMorph),\n";
        }

        content = content.replace("$BLOCKS", blocks_str.as_str());
        
        let mut label_parts_string = "".to_string();

        for label_part in label_parts.values() {
            label_parts_string += "\t\t\t\tnew Extension.LabelPart(\n";
            label_parts_string += format!("\t\t\t\t\t'{}',\n", label_part.spec).as_str();
            label_parts_string += "\t\t\t\t\t() => {\n";
            label_parts_string += "\t\t\t\t\t\tconst part = new InputSlotMorph(\n";
            label_parts_string += "\t\t\t\t\t\t\tnull, // text\n";
            label_parts_string += "\t\t\t\t\t\t\tfalse, // non-numeric\n";
            label_parts_string += "\t\t\t\t\t\t\tnull,\n";
            label_parts_string += "\t\t\t\t\t\t\tfalse\n";
            label_parts_string += "\t\t\t\t\t\t);\n";
            label_parts_string += "\t\t\t\t\t\treturn part;\n";
            label_parts_string += "\t\t\t\t\t}\n";
            label_parts_string += "\t\t\t\t),\n";
        }

        content = content.replace("$LABELPARTS", &label_parts_string);

        content = content.replace("$IMPORTS_LIST", &fn_names.join(", "));
        content = content.replace("$WINDOW_IMPORTS", &fn_names.iter().map(|fn_name| format!("\t\twindow.{}_fns.{} = {};", extension_info.name, fn_name, fn_name)).collect::<Vec<String>>().join("\n"));
        

        let mut out_file = File::create("./index.js")?;
        out_file.write_all(content.as_bytes())?;

    } else {
        bail!("No ExtensionInfo found!");
    }

    Ok(())
}