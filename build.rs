use std::{fs::File, error::Error, io::{Read, Write}, vec, collections::HashMap, hash::Hash};

use netsblox_extension_util::{ExtensionInfo, CustomBlock, BlockType, Category, TargetObject};
use regex::Regex;
use simple_error::bail;
use syn::{Item, PathSegment, ItemConst, Expr, Member, Lit};

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
                        "name" => { 
                            if let Expr::Lit(lit) = &field.expr {
                                if let Lit::Str(val) = &lit.lit {
                                    let mut lit = val.token().to_string();

                                    // Remove quotes from Literal value
                                    lit = lit.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap().to_string();

                                    // Leaking would be bad, but this script has a short life
                                    instance.name = Box::leak(lit.into_boxed_str());
                                }
                            }
                        }
                        _ => { warn!("Unknown field: {}", name) }
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
                        "name" => { 
                            instance.name = extract_string(field);
                        },
                        "block_type" => {
                            if let Expr::Path(p) = &field.expr {
                                let block_type = &p.path.segments.last().unwrap().ident.to_string();
                                match block_type.as_str() {
                                    "Command" => { instance.block_type = BlockType::Command },
                                    "Reporter" => { instance.block_type = BlockType::Reporter },
                                    "Predicate" => { instance.block_type = BlockType::Predicate },
                                    _ => {
                                        warn!("Unknown block type {}", block_type)
                                    }
                                }
                            }
                        },
                        "category" => {
                            if let Expr::Path(p) = &field.expr {
                                let cat = &p.path.segments.last().unwrap().ident.to_string();
                                match cat.as_str() {
                                    "Motion" => { instance.category = Category::Motion },
                                    "Looks" => { instance.category = Category::Looks },
                                    "Sound" => { instance.category = Category::Sound },
                                    "Pen" => { instance.category = Category::Pen },
                                    "Network" => { instance.category = Category::Network },
                                    "Control" => { instance.category = Category::Control },
                                    "Sensing" => { instance.category = Category::Sensing },
                                    "Operators" => { instance.category = Category::Operators },
                                    "Variables" => { instance.category = Category::Variables },
                                    "Custom" => { instance.category = Category::Custom },
                                    //TODO "CustomCategory" => { instance.category = Category::CustomCategory(())},
                                    _ => {
                                        warn!("Unknown category {}", cat)
                                    }
                                }
                            }
                        },
                        "spec" => { 
                            instance.spec = extract_string(field);

                        },
                        "defaults" => {
                            // TODO
                        },
                        "impl_fn" => { 
                            instance.impl_fn = extract_string(field);
                        },
                        _ => { warn!("Unknown field: {}", name) }
                    }
                }
            }
        }
    }
    instance
}

fn extract_string(field: &syn::FieldValue) -> &'static mut str {
    if let Expr::Lit(lit) = &field.expr {
        if let Lit::Str(val) = &lit.lit {
            let mut lit = val.token().to_string();

            // Remove quotes from Literal value
            lit = lit.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap().to_string();

            // Leaking would be bad, but this script has a short life
            return Box::leak(lit.into_boxed_str());
        }
    }
    
    return Box::leak("".to_string().into_boxed_str());
}

fn main() -> Result<(), Box<dyn Error>>  {  
    // Read file  
    let mut file = File::open("./src/lib.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let ast = syn::parse_file(&content)?;
    

    let mut extension_info: Option<ExtensionInfo> = None;
    let mut custom_blocks: HashMap<String, CustomBlock> = HashMap::new();

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
                        custom_blocks.insert(block.name.to_string(), block);
                    },
                    _ => {}
                };
            }
        }
    }

    if let Some(extension_info) = extension_info {
        let mut file = File::open("./template.js")?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        content = content.replace("$EXTENSION_NAME", extension_info.name);
        content = content.replace("$MENU", "");
        content = content.replace("$SETTINGS", "");
        content = content.replace("$CATEGORIES", "");

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

            blocks_str += format!("\t\t\t\t\tfunction ({}) {{ {}({}) }}\n", label_parts_str, block.impl_fn, label_parts_str).as_str();
            blocks_str += "\t\t\t\t).for(SpriteMorph, StageMorph),\n";
        }

        content = content.replace("$BLOCKS", blocks_str.as_str());
        
        content = content.replace("$LABELPARTS", "");

        let mut out_file = File::create("./index.js")?;
        out_file.write_all(content.as_bytes())?;

    } else {
        bail!("No ExtensionInfo found!");
    }

    Ok(())
}