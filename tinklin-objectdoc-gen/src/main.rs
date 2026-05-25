#![feature(string_replace_in_place)]

use serde::Deserialize;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;

type ObjectTypeDef = HashMap<String, ObjectField>;

#[derive(Clone, Debug, Deserialize)]
struct ObjectField(String, String);

#[derive(Debug, Deserialize)]
struct ObjectDefault {
    #[serde(rename = "_type")]
    typ: String,
    #[serde(flatten)]
    fields: HashMap<String, String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ObjectFieldData {
    name: String,
    typ: String,
    default_value: String,
    comment: String,
}

static CURRENT_DIR: LazyLock<Box<PathBuf>> =
    LazyLock::new(|| Box::new(std::env::current_dir().unwrap()));
static OUTPUT_DIR: LazyLock<Box<PathBuf>> = LazyLock::new(|| {
    let output_dir = CURRENT_DIR.join("generated");
    if !output_dir.exists() {
        fs::create_dir(output_dir.as_path()).unwrap();
    }
    Box::new(output_dir)
});

fn create_file(filename: &str) -> File {
    File::create(OUTPUT_DIR.join(filename)).unwrap()
}

fn open_file(filename: &str) -> File {
    File::open(CURRENT_DIR.join(filename)).unwrap()
}

fn assert_keys_match<K: Eq + Hash + Debug, V1, V2>(map_a: &HashMap<K, V1>, map_b: &HashMap<K, V2>) {
    let only_in_a: Vec<_> = map_a.keys().filter(|k| !map_b.contains_key(k)).collect();

    let only_in_b: Vec<_> = map_b.keys().filter(|k| !map_a.contains_key(k)).collect();

    if !only_in_a.is_empty() || !only_in_b.is_empty() {
        panic!(
            "Keys doesn't match\n    Keys missing in map_a: {:?}\n    Keys missing in map_B: {:?}",
            only_in_b, only_in_a
        );
    }
}

fn main() {
    let object_typ_defs: HashMap<String, ObjectTypeDef> = {
        let mut file = open_file("object_type_defs.toml");
        let mut object_typ_defs = String::new();
        file.read_to_string(&mut object_typ_defs).unwrap();
        toml::from_str(&object_typ_defs).unwrap()
    };
    let object_defaults: HashMap<String, ObjectDefault> = {
        let mut file = open_file("object_defaults.toml");
        let mut object_defaults = String::new();
        file.read_to_string(&mut object_defaults).unwrap();
        toml::from_str(&object_defaults).unwrap()
    };
    let object_data: HashMap<_, _> = object_defaults
        .into_iter()
        .map(
            |(
                mut object_name,
                ObjectDefault {
                    typ: object_typ,
                    fields,
                },
            )| {
                let mut typ_def = object_typ_defs
                    .get(&object_typ)
                    .unwrap_or_else(|| panic!("Unknown object type: {}", object_typ))
                    .clone();
                assert_keys_match(&fields, &typ_def);
                let field_data = fields
                    .into_iter()
                    .map(|(field_name, mut default_value)| {
                        let ObjectField(field_typ, comment) = typ_def.remove(&field_name).unwrap();
                        if field_typ == "string" {
                            default_value = format!("\"{}\"", default_value);
                        }
                        ObjectFieldData {
                            name: field_name,
                            typ: field_typ,
                            default_value,
                            comment,
                        }
                    })
                    .collect::<BTreeSet<_>>();
                object_name.replace_first("__", ".");
                (object_name, field_data)
            },
        )
        .collect();
    let mut output = create_file("objects.lua");
    writeln!(output, "---@meta").unwrap();
    writeln!(output, "").unwrap();
    for (object_name, field_data) in &object_data {
        writeln!(output, "---@class {}", object_name).unwrap();
        for ObjectFieldData {
            name,
            typ,
            default_value,
            comment,
        } in field_data
        {
            if typ == "unsupported" {
                continue;
            }
            let desc = {
                if comment.is_empty() {
                    format!("Default: {}", default_value)
                } else {
                    format!("Default: {}<br>{}", default_value, comment)
                }
            };
            writeln!(output, "---@field {} {} {}", name, typ, desc).unwrap();
        }
        writeln!(
            output,
            "---@field set fun(self, fields: {}.set)",
            object_name
        )
        .unwrap();
        writeln!(output, "").unwrap();
    }

    let mut output = create_file("object_set.lua");
    writeln!(output, "---@meta").unwrap();
    writeln!(output, "").unwrap();
    for (object_name, field_data) in &object_data {
        writeln!(output, "---@class {}.set", object_name).unwrap();
        for ObjectFieldData {
            name,
            typ,
            default_value,
            comment,
        } in field_data
        {
            if typ == "unsupported" {
                continue;
            }
            assert_ne!(
                default_value, "",
                "Missing default value for supported field: {}.{}",
                object_name, name
            );
            let desc = {
                if comment.is_empty() {
                    format!("Default: {}", default_value)
                } else {
                    format!("Default: {}<br>{}", default_value, comment)
                }
            };
            writeln!(output, "---@field {} {}? {}", name, typ, desc).unwrap();
        }
        writeln!(output, "").unwrap();
    }
}
