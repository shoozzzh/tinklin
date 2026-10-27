use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    sync::LazyLock,
};

struct CompDoc(Vec<Component>);

struct Component {
    name: String,
    fields: Vec<Field>,
}

struct Field {
    name: String,
    typ: (TypCategory, String),
    desc: String,
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

fn replace_non_legal_chars(s: String) -> String {
    s.chars()
        .map(|c| match c {
            ':' | '<' | '>' => '_',
            _ => c,
        })
        .collect()
}

static EXPECTED_IDX: &[usize; 3] = &[4, 28, 92];

fn parse_field_line(line: &str) -> (&str, &str, &str) {
    let overflows = line[EXPECTED_IDX[2]..]
        .find(|c: char| !c.is_whitespace())
        .unwrap();
    let idx_2 = EXPECTED_IDX[2] + overflows;
    let idx_1 = EXPECTED_IDX[1] + overflows;
    let idx_0 = EXPECTED_IDX[0];
    let typ = &line[idx_0..idx_1].trim_end();
    let name = &line[idx_1..idx_2].trim_end();
    let desc = &line[idx_2..].trim_end();
    (typ, name, desc)
}

static TYP_MAPPING: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("LensValue<bool>", "boolean"),
        ("int", "integer"),
        ("int16", "integer"),
        ("int32", "integer"),
        ("int64", "integer"),
        ("uint16", "integer"),
        ("uint32", "integer"),
        ("uint32_t", "integer"),
        ("uint64", "integer"),
        ("unsigned int", "integer"),
        ("LensValue<int>", "integer"),
        ("EntityID", "entity_id"),
        ("EntityTypeID", "integer"),
        ("bool", "boolean"),
        ("float", "number"),
        ("double", "number"),
        ("LensValue<float>", "number"),
        ("vec2", "vector"),
        ("ivec2", "vector"),
        ("Vec2ArrayInline", "vector[]"),
        ("ENTITY_VEC", "entity_id[]"),
        ("VEC_ENTITY", "entity_id[]"),
        ("VECTOR_ENTITYID", "entity_id[]"),
        ("std::vector<int>", "integer[]"),
        ("std::vector< int >", "integer[]"),
        ("VECTOR_INT", "integer[]"),
        ("VECTOR_INT32", "integer[]"),
        ("VEC_OF_MATERIALS", "material_id[]"),
        ("std::vector< float >", "number[]"),
        ("VECTOR_FLOAT", "number[]"),
        ("FloatArrayInline", "number[]"),
        ("std::string", "string"),
        ("std_string", "string"),
        ("USTRING", "string"),
        ("VECTOR_STR", "string[]"),
        ("VECTOR_STRING", "string[]"),
        ("VISITED_VEC", "string[]"),
        ("DAMAGE_TYPES::Enum", "damage_type"),
        ("HIT_EFFECT::Enum", "hit_effect"),
        ("GAME_EFFECT::Enum", "game_effect"),
        ("RAGDOLL_FX::Enum", "ragdoll_fx"),
        ("JOINT_TYPE::Enum", "joint_type"),
        ("ARC_TYPE::Enum", "arc_type"),
        ("VERLET_TYPE::Enum", "verlet_type"),
        ("EXPLOSION_TRIGGER_TYPE::Enum", "explosion_trigger_type"),
        (
            "PARTICLE_EMITTER_CUSTOM_STYLE::Enum",
            "particle_emitter_custom_style",
        ),
        ("INVENTORY_KIND::Enum", "inventory_kind"),
        ("LUA_VM_TYPE::Enum", "lua_vm_type"),
        ("PROJECTILE_TYPE::Enum", "projectile_type"),
        ("MOVETOSURFACE_TYPE::Enum", "movetosurface_type"),
        ("types::xform", "comp.xform"),
        ("types::aabb", "aabb"),
        ("types::iaabb", "aabb"),
        ("types::fcolor", "rgba_percent"),
        ("MATERIAL_VEC_DOUBLES", "table<material_id,number>"),
        ("ValueRange", "interval"),
        ("ValueRangeInt", "interval"),
    ])
});

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum TypCategory {
    Normal,
    Proxy,
    Object(String),
    ReadOnly,
    Unsupported,
}

impl TypCategory {
    fn figure_out(typ: &str, comp_name: &str, field_name: &str) -> (Self, String) {
        match TYP_MAPPING.get(typ).map(|s| *s) {
            Option::None => {
                if typ.starts_with("Config") {
                    (
                        Self::Object(typ.to_owned()),
                        format!("{}.{}", comp_name, field_name),
                    )
                } else {
                    (Self::Unsupported, replace_non_legal_chars(typ.to_owned()))
                }
            }
            Option::Some(typ_lua) => {
                if typ_lua == "comp.xform" {
                    (Self::Proxy, typ_lua.to_owned())
                } else if typ_lua.chars().any(|c| "[]<>".contains(c)) {
                    (Self::ReadOnly, typ_lua.to_owned())
                } else {
                    (Self::Normal, typ_lua.to_owned())
                }
            }
        }
    }
}

fn main() {
    let file = std::env::args().nth(1).expect("no filepath given");
    let file = File::open(file).expect("Unable to open file");
    let reader = BufReader::new(file);

    let mut comp_doc = CompDoc(Vec::new());

    let mut cur_comp: Option<Component> = Option::None;

    let mut typs_unsupported = BTreeSet::new();

    for (line_idx, line) in reader.lines().enumerate() {
        let line = line.expect("Unable to read line");
        if line.len() == 0 {
            comp_doc.0.push(cur_comp.unwrap());
            cur_comp = Option::None;
            continue;
        }
        let space_count = line.len() - line.trim_start().len();
        match space_count {
            0 => match cur_comp {
                Option::None => {
                    cur_comp = Option::Some(Component {
                        name: line,
                        fields: Vec::new(),
                    });
                }
                Option::Some(_) => {
                    panic!("unexpected comp start at line {}", line_idx)
                }
            },
            1 => {
                continue;
            }
            4 => {
                let (typ, name, desc) = parse_field_line(&line);
                if let Some(ref mut cur_comp) = cur_comp {
                    let name = name.to_owned();
                    let mut desc = {
                        let desc = desc
                            .split_whitespace()
                            .filter_map(|s| {
                                if s == "\"\"" {
                                    Option::None
                                } else {
                                    if s.starts_with('"') {
                                        Option::Some(&s[1..])
                                    } else if s.ends_with('"') {
                                        Option::Some(&s[..s.len() - 1])
                                    } else {
                                        Option::Some(s)
                                    }
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        match desc.as_str() {
                            "-" => "".to_owned(),
                            _ => desc,
                        }
                    };
                    let typ = TypCategory::figure_out(typ, &cur_comp.name, &name);
                    match typ.0 {
                        TypCategory::ReadOnly => {
                            desc = "[Field is read-only]<br>".to_owned() + &desc;
                        }
                        TypCategory::Unsupported => {
                            typs_unsupported.insert(typ.1.clone());
                        }
                        _ => {}
                    }
                    cur_comp.fields.push(Field { name, typ, desc });
                }
            }
            other => {
                panic!("unexpected space count: {}, at line {}", other, line_idx)
            }
        }
    }

    let mut output = create_file("comps.lua");

    writeln!(output, "---@meta").unwrap();
    writeln!(output, "").unwrap();
    for comp in &comp_doc.0 {
        writeln!(output, "---@class {}: comp", comp.name).unwrap();
        comp.fields
            .iter()
            .filter(|field| {
                matches!(
                    field.typ.0,
                    TypCategory::Normal | TypCategory::ReadOnly | TypCategory::Proxy
                )
            })
            .for_each(|Field { name, typ, desc }| {
                if desc.is_empty() {
                    writeln!(output, "---@field {} {}", name, typ.1).unwrap();
                } else {
                    writeln!(output, "---@field {} {} {}", name, typ.1, desc).unwrap();
                }
            });
        writeln!(output, "local {} = {{}}", comp.name).unwrap();
        writeln!(output, "").unwrap();
        if comp
            .fields
            .iter()
            .any(|field| matches!(field.typ.0, TypCategory::Normal))
        {
            writeln!(output, "---@param fields {}.set", comp.name).unwrap();
            writeln!(output, "function {}:set(fields) end", comp.name).unwrap();
        } else {
            writeln!(output, "---This component doesn't have anything to set",).unwrap();
            writeln!(output, "---@param fields unsupported").unwrap();
            writeln!(output, "function {}:set(fields) end", comp.name).unwrap();
        }
        comp.fields
            .iter()
            .filter(|field| matches!(field.typ.0, TypCategory::Object(_)))
            .for_each(|field| {
                writeln!(output, "").unwrap();
                if !field.desc.is_empty() {
                    writeln!(output, "---{}", field.desc).unwrap();
                }
                writeln!(output, "---@param name \"{}\"", field.name).unwrap();
                writeln!(output, "---@return {}", field.typ.1).unwrap();
                writeln!(output, "function {}:object(name) end", comp.name).unwrap();
            });
        writeln!(output, "").unwrap();
    }
    writeln!(output, "---@class unsupported").unwrap();
    writeln!(output, "").unwrap();
    for typ in typs_unsupported {
        writeln!(output, "---@alias {} unsupported", typ).unwrap();
    }

    let mut output = create_file("comp_set.lua");
    for comp in &comp_doc.0 {
        let mut is_first_field = true;
        comp.fields
            .iter()
            .filter(|field| matches!(field.typ.0, TypCategory::Normal))
            .for_each(|field| {
                if is_first_field {
                    is_first_field = false;
                    writeln!(output, "---@class {}.set", comp.name).unwrap();
                }
                if field.desc.is_empty() {
                    writeln!(output, "---@field {} {}?", field.name, field.typ.1).unwrap();
                } else {
                    writeln!(
                        output,
                        "---@field {} {}? {}",
                        field.name, field.typ.1, field.desc
                    )
                    .unwrap();
                }
            });
        if is_first_field {
            writeln!(output, "---@alias {}.set unsupported", comp.name).unwrap();
        }
        writeln!(output, "").unwrap();
    }

    let mut output = create_file("comp_getters.lua");
    let template = BufReader::new(open_file("template_comp_getters.lua"));

    for line in template.lines().map(|l| l.unwrap()) {
        if line.starts_with("--@") {
            for comp in &comp_doc.0 {
                writeln!(output, "-{}", line.replace("{}", &comp.name)).unwrap();
            }
            continue;
        }
        writeln!(output, "{}", line).unwrap();
    }

    let mut output = create_file("fetch_object_fields.lua");
    let template = {
        let mut content = String::new();
        open_file("template_fetch_object_fields.lua")
            .read_to_string(&mut content)
            .unwrap();
        content
    };

    writeln!(output, "local objects = {{").unwrap();
    for comp in &comp_doc.0 {
        comp.fields.iter().for_each(|field| {
            if let TypCategory::Object(object_typ) = &field.typ.0 {
                writeln!(
                    output,
                    "    {{\"{}\", \"{}\", \"{}\"}},",
                    comp.name, field.name, object_typ,
                )
                .unwrap();
            }
        })
    }
    writeln!(output, "}}").unwrap();
    write!(output, "{}", template).unwrap();
}
