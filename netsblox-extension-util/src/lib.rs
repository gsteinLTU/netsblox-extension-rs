use proc_macro2::TokenTree;
use serde::Serialize;
use std::{fs::File, error::Error, io::{Read, Write}, vec, collections::{HashMap, HashSet}, path::Path, fmt::Write as FmtWrite};
use regex::Regex;
use simple_error::bail;
use syn::{Attribute, Expr, ExprCall, ExprPath, ExprLit, ExprReference, ExprArray, ExprStruct, Lit, Item, ItemConst, ItemFn, Member, Meta, PathSegment};
use std::collections::BTreeSet;

macro_rules! count_exprs {
    () => { 0usize };
    ($h:expr $(,$t:expr)*$(,)?) => { 1usize + count_exprs!($($t),*) };
}
macro_rules! extract_tuple {
    ($e:expr, $($parser:expr),*$(,)?) => {{
        match $e {
            Expr::Tuple(t) => {
                let n = count_exprs!($($parser),*);
                assert_eq!(n, t.elems.len(), "incorrect number of values in tuple");
                let mut v = t.elems.iter();
                ($($parser(v.next().unwrap())),*)
            }
            x => panic!("unknown tuple expr: {x:?}"),
        }
    }};
}

macro_rules! try_construct {
    ($t:ident$(::$tt:ident)* { $($f:ident),*$(,)? }) => {
        $t$(::$tt)* { $($f: $f.expect(&format!("missing {} field: {}", stringify!($t), stringify!($f)))),* }
    };
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ExtensionInfo {
    pub name: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomBlock {
    pub name: &'static str,
    pub block_type: BlockType,
    pub category: &'static str,
    pub spec: &'static str,
    pub defaults: &'static str,
    pub impl_fn: &'static str,
    pub target: TargetObject,
    pub pass_proc: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum TargetObject {
    SpriteMorph, StageMorph, Both,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum BlockType {
    #[serde(rename = "command")]
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

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Menu {
    Entry {
        label: &'static str,
        value: &'static str,
    },
    Submenu {
        label: &'static str,
        content: &'static [Menu],
    },
}

#[derive(Debug,Clone, Copy, Serialize)]
pub struct LabelPart {
    pub spec: &'static str,
    pub text: Option<&'static str>,
    pub numeric: bool,
    pub menu: Option<&'static [Menu]>,
    pub readonly: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct CustomCategory {
    pub name: &'static str,
    pub color: (f64, f64, f64)
}

#[derive(Debug, Clone, Copy, Serialize)]
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

fn format_menu(menu: &[Menu]) -> String {
    fn visitor(menu: &Menu, res: &mut String) {
        match menu {
            Menu::Entry { label, value } => write!(res, "{label:?}: {value:?},").unwrap(),
            Menu::Submenu { label, content } => {
                write!(res, "{label:?}: {{").unwrap();
                for x in *content {
                    visitor(x, res);
                }
                res.push_str("},");
            }
        }
    }

    let mut res = String::new();
    res.push('{');
    for x in menu {
        visitor(x, &mut res);
    }
    res.push('}');
    res
}

// Turn syn item into instance
fn recreate_netsblox_extension_info(item: &ItemConst) -> ExtensionInfo {
    let mut name: Option<&'static str> = None;

    match &*item.expr {
        Expr::Struct(s) => {
            for field in &s.fields {
                match &field.member {
                    Member::Named(named) => match named.to_string().as_str() {
                        "name" => name = Some(extract_string(&field.expr)),
                        x => panic!("unknown extension info field: {x:?}"),
                    }
                    x => panic!("unknown extension info field member: {x:?}"),
                }
            }
        }
        x => panic!("unknown extension info expr: {x:?}"),
    }

    try_construct!(ExtensionInfo { name })
}

// Turn syn item into instance
fn recreate_netsblox_extension_custom_category(item: &ItemConst) -> CustomCategory {
    let mut name: Option<&'static str> = None;
    let mut color: Option<(f64, f64, f64)> = None;

    match &*item.expr {
        Expr::Struct(s) => {
            for field in &s.fields {
                match &field.member {
                    Member::Named(named) => match named.to_string().as_str() {
                        "name" => name = Some(extract_string(&field.expr)),
                        "color" => color = Some(extract_tuple!(&field.expr, extract_f64, extract_f64, extract_f64)),
                        x => panic!("unknown custom category field: {x:?}"),
                    }
                    x => panic!("unknown custom category field member: {x:?}"),
                }
            }
        }
        x => panic!("unknown custom category expr: {x:?}"),
    }

    try_construct!(CustomCategory { name, color })
}

// Turn syn item into instance
fn recreate_netsblox_extension_block(item: &ItemFn, attr: &Attribute) -> CustomBlock {
    let mut name: Option<&'static str> = None;
    let mut category: Option<&'static str> = None;
    let mut spec: Option<&'static str> = None;
    let mut defaults: Option<&'static str> = None;
    let mut target: Option<TargetObject> = None;
    let mut pass_proc: Option<bool> = None;
    let mut impl_fn: Option<&'static str> = None;
    let mut block_type: Option<BlockType> = None;

    // Parse information stored in attribute
    if let Meta::List(l) = &attr.meta {
        let t = &l.tokens.clone().into_iter().collect::<Vec<_>>();

        for arg in t.split(|x| matches!(x, TokenTree::Punct(p) if p.as_char() == ',')) {
            match arg {
                [TokenTree::Ident(attr_name), TokenTree::Punct(eq), value @ ..] if eq.as_char() == '=' => match attr_name.to_string().as_str() {
                    "name" => name = Some(extract_string_meta(value)),
                    "category" => category = Some(extract_string_meta(value)),
                    "spec" => spec = Some(extract_string_meta(value)),
                    "defaults" => defaults = Some(extract_string_meta(value)),
                    "pass_proc" => pass_proc = Some(extract_bool_meta(value)),
                    "type_override" | "block_type" => block_type = Some(extract_block_type_meta(value)), // Allows for overriding block types if desired, or to make hat/terminal blocks possible
                    "target" => target = Some(extract_target_object_meta(value)),
                    x => panic!("unknown extension block attr field: {x:?}"),
                }
                x => panic!("unknown meta attr format: {x:?}"),
            }
        }
    }

    if defaults.is_none() {
        defaults = Some("[]");
    }
    if target.is_none() {
        target = Some(TargetObject::Both);
    }
    if pass_proc.is_none() {
        pass_proc = Some(false);
    }
    if impl_fn.is_none() {
        impl_fn = Some(Box::leak(item.sig.ident.to_string().into_boxed_str())); // Get information from function signature
    }
    if block_type.is_none() {
        block_type = Some(match &item.sig.output {
            syn::ReturnType::Default => BlockType::Command,
            syn::ReturnType::Type(_, b) => match b.as_ref() {
                syn::Type::Tuple(t) if t.elems.is_empty() => BlockType::Command,
                syn::Type::Path(p) if p.path.segments.first().unwrap().ident.to_string() == "bool" => BlockType::Predicate,
                syn::Type::Path(p) if p.path.segments.first().unwrap().ident.to_string() == "Result" => match &p.path.segments.first().unwrap().arguments {
                    syn::PathArguments::AngleBracketed(x) => match x.args.first().unwrap() {
                        syn::GenericArgument::Type(c) => match c {
                            syn::Type::Tuple(t) if t.elems.is_empty() => BlockType::Command,
                            syn::Type::Path(p) if p.path.segments.first().unwrap().ident.to_string() == "bool" => BlockType::Predicate,
                            _ => BlockType::Reporter
                        }
                        _ => BlockType::Reporter
                    }
                    _ => BlockType::Reporter
                }
                _ => BlockType::Reporter
            },
        });
    }

    try_construct!(CustomBlock { name, block_type, category, spec, defaults, impl_fn, target, pass_proc })
}

// Turn syn item into instance
fn recreate_netsblox_extension_label_part(item: &ItemConst) -> LabelPart {
    let mut spec: Option<&'static str> = None;
    let mut text: Option<Option<&'static str>> = None;
    let mut menu: Option<Option<&[Menu]>> = None;
    let mut numeric: Option<bool> = None;
    let mut readonly: Option<bool> = None;

    match &*item.expr {
        Expr::Struct(s) => {
            for field in &s.fields {
                match &field.member {
                    Member::Named(name) => match name.to_string().as_str() {
                        "spec" => spec = Some(extract_string(&field.expr)),
                        "text" => text = Some(extract_option(&field.expr, extract_string)),
                        "numeric" => numeric = Some(extract_bool(&field.expr)),
                        "menu" => menu = Some(extract_option(&field.expr, |x| extract_slice(x, &extract_menu))),
                        "readonly" => readonly = Some(extract_bool(&field.expr)),
                        x => panic!("unknown label part field: {x}"),
                    }
                    x => panic!("unknown label part field member: {x:?}"),
                }
            }
        }
        x => panic!("unknown label part expr: {x:?}"),
    }

    try_construct!(LabelPart { spec, text, numeric, menu, readonly })
}

fn recreate_netsblox_extension_setting(item: &ItemConst) -> ExtensionSetting {
    let mut name: Option<&'static str> = None;
    let mut id: Option<&'static str> = None;
    let mut default_value: Option<bool> = None;
    let mut on_hint: Option<&'static str> = None;
    let mut off_hint: Option<&'static str> = None;
    let mut hidden: Option<bool> = None;

    match &*item.expr {
        Expr::Struct(s) => {
            for field in s.fields.iter() {
                match &field.member {
                    Member::Named(named) => match named.to_string().as_str() {
                        "name" => name = Some(extract_string(&field.expr)),
                        "id" => id = Some(extract_string(&field.expr)),
                        "default_value" => default_value = Some(extract_bool(&field.expr)),
                        "on_hint" => on_hint = Some(extract_string(&field.expr)),
                        "off_hint" => off_hint = Some(extract_string(&field.expr)),
                        "hidden" => hidden = Some(extract_bool(&field.expr)),
                        x => panic!("unknown extension setting field: {x:?}"),
                    }
                    x => panic!("unknown extension setting field member: {x:?}"),
                }
            }
        }
        x => panic!("unknown extension setting expr: {x:?}"),
    }

    try_construct!(ExtensionSetting { name, id, default_value, on_hint, off_hint, hidden })
}

fn extract_menu(expr: &Expr) -> Menu {
    match expr {
        Expr::Struct(ExprStruct { attrs: _, qself: _, path, brace_token: _, fields, dot2_token: _, rest: _ }) if path.segments.len() == 2 && path.segments.first().unwrap().ident.to_string() == "Menu" => match path.segments.last().unwrap().ident.to_string().as_str() {
            "Entry" => {
                let mut label: Option<&'static str> = None;
                let mut value: Option<&'static str> = None;

                for field in fields {
                    match &field.member {
                        Member::Named(name) if name.to_string() == "label" => label = Some(extract_string(&field.expr)),
                        Member::Named(name) if name.to_string() == "value" => value = Some(extract_string(&field.expr)),
                        x => panic!("unknown menu entry field: {x:?}"),
                    }
                }

                try_construct!(Menu::Entry { label, value })
            }
            "Submenu" => {
                let mut label: Option<&'static str> = None;
                let mut content: Option<&'static [Menu]> = None;

                for field in fields {
                    match &field.member {
                        Member::Named(name) if name.to_string() == "label" => label = Some(extract_string(&field.expr)),
                        Member::Named(name) if name.to_string() == "content" => content = Some(extract_slice(&field.expr, &extract_menu)),
                        x => panic!("unknown menu submenu field: {x:?}"),
                    }
                }

                Menu::Submenu {
                    label: label.expect("missing menu submenu label field"),
                    content: content.expect("missing menu submenu content field"),
                }
            }
            x => panic!("unknown menu variant: {x:?}"),
        }
        x => panic!("unknown menu expr: {x:?}"),
    }
}

fn extract_block_type_meta(tree: &[TokenTree]) -> BlockType {
    match tree {
        [.., TokenTree::Ident(t), TokenTree::Punct(c1), TokenTree::Punct(c2), TokenTree::Ident(v)] if t.to_string() == "BlockType" && c1.as_char() == ':' && c2.as_char() == ':' => match v.to_string().as_str() {
            "Terminator" => BlockType::Terminator,
            "Command" => BlockType::Command,
            "Reporter" => BlockType::Reporter,
            "Predicate" => BlockType::Predicate,
            "Hat" => BlockType::Hat,
            x => panic!("unknown block type variant: {x:?}"),
        }
        x => panic!("unknown block type meta expr: {x:?}"),
    }
}
fn extract_target_object_meta(tree: &[TokenTree]) -> TargetObject {
    match tree {
        [.., TokenTree::Ident(t), TokenTree::Punct(c1), TokenTree::Punct(c2), TokenTree::Ident(v)] if t.to_string() == "TargetObject" && c1.as_char() == ':' && c2.as_char() == ':' => match v.to_string().as_str() {
            "Both" => TargetObject::Both,
            "SpriteMorph" => TargetObject::SpriteMorph,
            "StageMorph" => TargetObject::StageMorph,
            x => panic!("unknown block type variant: {x:?}"),
        }
        x => panic!("unknown block type meta expr: {x:?}"),
    }
}

fn extract_option<T, F: FnOnce(&Expr) -> T>(expr: &Expr, parser: F) -> Option<T> {
    match expr {
        Expr::Call(ExprCall { attrs: _, func, paren_token: _, args }) => match &**func {
            Expr::Path(ExprPath { attrs: _, qself: _, path }) if path.segments.len() == 1 && path.segments.first().unwrap().ident.to_string() == "Some" && args.len() == 1 => Some(parser(args.first().unwrap())),
            x => panic!("unknown option call expr: {x:?}"),
        }
        Expr::Path(ExprPath { attrs: _, qself: _, path }) if path.segments.len() == 1 && path.segments.first().unwrap().ident.to_string() == "None" => None,
        x => panic!("unknown option expr: {x:?}"),
    }
}

fn extract_slice<T, F: Fn(&Expr) -> T>(expr: &Expr, parser: &F) -> &'static [T] {
    match expr {
        Expr::Reference(ExprReference { attrs: _, and_token: _, mutability: _, expr }) => match &**expr {
            Expr::Array(ExprArray { attrs: _, bracket_token: _, elems }) => elems.iter().map(parser).collect::<Vec<_>>().leak(),
            x => panic!("unknown slice ref expr: {x:?}"),
        }
        x => panic!("unknown slice expr: {x:?}"),
    }
}

fn extract_string(expr: &syn::Expr) -> &'static str {
    match expr {
        Expr::Lit(ExprLit { attrs: _, lit: Lit::Str(v) }) => v.value().leak(), // Leaking would be bad, but this script has a short life
        x => panic!("unknown string expr: {x:?}"),
    }
}
fn extract_string_meta(tree: &[TokenTree]) -> &'static str {
    match tree {
        [TokenTree::Literal(lit)] => {
            let v = lit.to_string();
            v[v.find('"').unwrap() + 1..v.rfind('"').unwrap()].to_owned().leak()
        }
        x => panic!("unknown string meta value: {x:?}"),
    }
}

fn extract_bool(expr: &syn::Expr) -> bool {
    match expr {
        Expr::Lit(ExprLit { attrs: _, lit: Lit::Bool(v) }) => v.value,
        x => panic!("unknown bool expr: {x:?}"),
    }
}
fn extract_bool_meta(tree: &[TokenTree]) -> bool {
    match tree {
        [x] if x.to_string() == "true" => true,
        [x] if x.to_string() == "false" => false,
        x => panic!("unknown bool meta value: {x:?}"),
    }
}

fn extract_f64(expr: &syn::Expr) -> f64 {
    match expr {
        Expr::Lit(ExprLit { attrs: _, lit: Lit::Float(v) }) => v.base10_parse().unwrap(),
        x => panic!("unknown f64 expr: {x:?}"),
    }
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
            label_parts_string += format!("\t\t\t\t\t\t\t{}, // text\n", label_part.text.map(|x| format!("{x:?}")).unwrap_or_else(|| "null".into())).as_str();
            label_parts_string += format!("\t\t\t\t\t\t\t{}, // numeric\n", label_part.numeric).as_str();
            label_parts_string += format!("\t\t\t\t\t\t\t{}, // options\n", label_part.menu.map(|x| format_menu(x)).unwrap_or_else(|| "null".into())).as_str();
            label_parts_string += format!("\t\t\t\t\t\t\t{} // readonly\n", label_part.readonly).as_str();
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