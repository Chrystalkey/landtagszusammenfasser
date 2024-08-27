use std::char;
use std::collections::HashMap;
use std::io::Write;
use std::slice::ChunkBy;

use clap::{Parser, ValueEnum};
use sqlparser::ast;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser as SQLParser;

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum DatabaseType {
    Postgres,
}

#[derive(Parser)]
#[command(version, about, long_about= None)]
struct InputOptions {
    input_files: Vec<String>,

    #[arg(long, short, default_value_t = false)]
    verbose: bool,

    #[arg(long, short, default_value_t=DatabaseType::Postgres, value_enum)]
    format: DatabaseType,

    #[arg(long, short, default_value_t = String::from("data-structure-output.rs"))]
    output: String,
}

fn main() {
    let opts = InputOptions::parse();
    if opts.input_files.is_empty() {
        eprintln!("No input files provided");
        std::process::exit(1);
    }
    let mut sql = std::fs::read_to_string(&opts.input_files[0]).unwrap();
    if opts.input_files.len() > 1 {
        for i in 1..opts.input_files.len() {
            sql.push_str(&std::fs::read_to_string(&opts.input_files[i]).unwrap());
        }
    }

    let ast = SQLParser::parse_sql(&PostgreSqlDialect {}, &sql).unwrap();
    //println!("AST: {:#?}", ast);
    println!("Length: {}", ast.len());
    generate_output_file(&opts, ast);
}
struct RustStructField {
    name: String,
    rust_type: String,
}
struct RustStruct {
    name: String,
    fields: Vec<RustStructField>,
}
impl RustStruct {
    fn new(name: String) -> Self {
        RustStruct {
            name: name,
            fields: vec![],
        }
    }
}
impl From<RustStruct> for String {
    fn from(value: RustStruct) -> Self {
        let mut output = String::new();
        output.push_str(&format!("struct {}{{\n", value.name));
        for field in value.fields {
            output.push_str(&format!("\tpub {}: {},\n", field.name, field.rust_type));
        }
        output.push_str("}\n");
        output
    }
}

enum Relationship {
    ListEntity(String),
    Entity,
    NToN,
}

/// A table is a
/// - list entity, if it has exactly one ON DELETE CASCADE constraint
/// - entity, if it has no ON DELETE CASCADE
/// - NToN if it has at least two ON DELETE CASCADE constraints
fn classify(table: &ast::CreateTable) -> Relationship {
    let mut on_del_cascade = vec![];
    for column in &table.columns {
        for option in &column.options {
            match option {
                ast::ColumnOptionDef { option, .. } => match option {
                    ast::ColumnOption::ForeignKey {
                        on_delete,
                        foreign_table,
                        ..
                    } => {
                        if on_delete.is_some()
                            && on_delete.unwrap() == ast::ReferentialAction::Cascade
                        {
                            on_del_cascade.push(foreign_table.0.iter().fold(
                                String::new(),
                                |mut acc, x| {
                                    acc.push_str(&x.value);
                                    acc
                                },
                            ));
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    match on_del_cascade.len() {
        0 => Relationship::Entity,
        1 => Relationship::ListEntity(on_del_cascade.pop().unwrap()),
        _ => Relationship::NToN,
    }
}
fn split_names(name: &str) -> Vec<&str> {
    let chunks: Vec<&str> = name
        .split(|p| p == '_' || p == '-' || p == ' ' || p == '\t')
        .collect();
    chunks
        .iter()
        .map(|&chunk| {
            let mut subchunks = vec![];
            let mut last_pos = 1;
            while let Some(pos) = chunk[last_pos..].chars().position(|c| c.is_uppercase()) {
                let adjusted_pos = pos + last_pos;
                subchunks.push(&chunk[(last_pos - 1)..(adjusted_pos)]);
                last_pos = adjusted_pos + 1;
            }
            subchunks.push(&chunk[last_pos - 1..]);
            subchunks
        })
        .flatten()
        .collect::<Vec<&str>>()
}

#[test]
fn test_namesplit() {
    let names = vec![
        "abc-xyz-ghf",
        "abc_xyz_ghf",
        "abc xyz ghf",
        "AbcXyzGhf",
        "abcXyzGhf",
        "abc_xyzGhf",
        "abc-xyzGhf",
        "abc_xyz-ghf",
        "abc-xyz_Ghf",
        "abc_Xyz-ghf",
    ];
    println!("Testing namesplit");
    for name in names {
        println!("Testing: {}", name);
        let result = split_names(name);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].to_lowercase(), "abc");
        assert_eq!(result[1].to_lowercase(), "xyz");
        assert_eq!(result[2].to_lowercase(), "ghf");
    }
}

fn camelcase(name: &str) -> String {
    let mut chunks = split_names(name);
    chunks
        .iter()
        .map(|&chunk| {
            let mut chars = chunk.chars();
            let first = chars.next().unwrap().to_uppercase().collect::<String>();
            let rest = chars.collect::<String>();
            format!("{}{}", first, rest)
        })
        .fold(String::new(), |mut acc, x| {
            acc.push_str(&x);
            acc
        })
}

fn snake_case(name: &str) -> String {
    let mut chunks = split_names(name);
    let mut output = String::new();
    for chunk in &mut chunks {
        output.push_str(&chunk.to_lowercase());
        output.push('_');
    }
    output[0..output.len() - 1].to_string()
}

fn sql2rstype(sql_type: &ast::DataType) -> String {
    match sql_type {
        ast::DataType::Bool | ast::DataType::Boolean => "bool",
        ast::DataType::Int128 => "i128",
        ast::DataType::BigInt(_) => "i64",
        ast::DataType::Int(_) | ast::DataType::MediumInt(_) => "i32",
        ast::DataType::SmallInt(_) => "i16",
        ast::DataType::TinyInt(_) => "i8",
        ast::DataType::Float32 | ast::DataType::Float(_) | ast::DataType::Numeric(_) => "f32",
        ast::DataType::Double | ast::DataType::Real | ast::DataType::BigDecimal(_) => "f64",
        ast::DataType::Text
        | ast::DataType::Char(_)
        | ast::DataType::Varchar(_)
        | ast::DataType::CharacterVarying(_)
        | ast::DataType::Character(_) => "String",
        ast::DataType::Date | ast::DataType::Datetime(_) | ast::DataType::Timestamp(_, _) => {
            "chrono::DateTime<chrono::Utc>"
        }
        ast::DataType::Uuid => "uuid::Uuid",
        ast::DataType::UInt8 | ast::DataType::UnsignedTinyInt(_) => "u8",
        ast::DataType::UInt16
        | ast::DataType::UnsignedSmallInt(_)
        | ast::DataType::UnsignedInt2(_) => "u16",
        ast::DataType::UInt32 | ast::DataType::UnsignedInt(_) | ast::DataType::UnsignedInt4(_) => {
            "u32"
        }
        ast::DataType::UInt64
        | ast::DataType::UnsignedBigInt(_)
        | ast::DataType::UnsignedInt8(_) => "u64",
        ast::DataType::UInt128 => "u128",
        ast::DataType::Custom(name, _) => {
            if *name == ast::ObjectName(vec![ast::Ident::new("UUID")]) {
                "uuid::Uuid"
            } else if *name == ast::ObjectName(vec![ast::Ident::new("SERIAL")]) {
                "i32"
            } else {
                "String"
            }
        }
        _ => {
            unimplemented!("SQL type not supported: {:?}", sql_type)
        }
    }
    .to_owned()
}

fn generate_entity(ct: &ast::CreateTable) -> RustStruct {
    let mut rs_struct = RustStruct::new(camelcase(&ct.name.to_string()));
    for column in &ct.columns {
        let mut not_null = false;
        match column {
            ast::ColumnDef {
                name,
                data_type,
                options,
                ..
            } => {
                for option in options {
                    match option {
                        ast::ColumnOptionDef { option, .. } => match option {
                            ast::ColumnOption::NotNull => not_null = true,
                            ast::ColumnOption::Unique { is_primary, .. } => {
                                not_null = not_null || *is_primary;
                            }
                            _ => {}
                        },
                    }
                }
                rs_struct.fields.push(RustStructField {
                    name: snake_case(&name.value),
                    rust_type: if not_null {
                        sql2rstype(data_type)
                    } else {
                        format!("Option<{}>", sql2rstype(data_type))
                    },
                })
            }
        }
    }
    rs_struct
}

const PRESTRUCT_TEMPLATE: &str =
    "#[derive(Queryable, Insertable, Selectable, Debug, Default, Clone, Builder)]\n
#[diesel(table_name={})]\n
#[diesel(check_for_backend(diesel::pg::Pg))]";
const IMPL_DBCONNECTION : &str = 

fn generate_db_entity(ct: &ast::CreateTable) -> RustStruct {
    let mut name = camelcase(&ct.name.to_string());
    name.push_str("DB");
    let mut rs_struct = RustStruct::new(name);
    for column in &ct.columns {
        let mut not_null = false;
        match column {
            ast::ColumnDef {
                name,
                data_type,
                options,
                ..
            } => {
                for option in options {
                    match option {
                        ast::ColumnOptionDef { option, .. } => match option {
                            ast::ColumnOption::NotNull => not_null = true,
                            ast::ColumnOption::Unique { is_primary, .. } => {
                                not_null = not_null || *is_primary;
                            }
                            _ => {}
                        },
                    }
                }
                rs_struct.fields.push(RustStructField {
                    name: snake_case(&name.value),
                    rust_type: if not_null {
                        sql2rstype(data_type)
                    } else {
                        format!("Option<{}>", sql2rstype(data_type))
                    },
                })
            }
        }
    }
    rs_struct
}
fn generate_list_entity(
    ct: &ast::CreateTable,
    generated_tables: &mut HashMap<String, RustStruct>,
) -> RustStruct {
    let mut rs_struct = RustStruct::new(camelcase(&ct.name.to_string()));
    for column in &ct.columns {
        let mut not_null = false;
        let mut on_del_casc = false;
        let mut ref_table = None;

        match column {
            ast::ColumnDef {
                name,
                data_type,
                options,
                ..
            } => {
                for option in options {
                    match option {
                        ast::ColumnOptionDef { option, .. } => match option {
                            ast::ColumnOption::NotNull => not_null = true,
                            ast::ColumnOption::ForeignKey {
                                foreign_table,
                                on_delete,
                                ..
                            } => {
                                if *on_delete == Some(ast::ReferentialAction::Cascade) {
                                    ref_table = Some(foreign_table.0.iter().fold(
                                        String::new(),
                                        |mut acc, x| {
                                            acc.push_str(&x.value);
                                            acc
                                        },
                                    ));
                                    on_del_casc = true;
                                    not_null = true;
                                }
                            }
                            ast::ColumnOption::Unique { is_primary, .. } => {
                                not_null = not_null || *is_primary;
                            }
                            _ => {}
                        },
                    }
                }
                if !on_del_casc {
                    rs_struct.fields.push(RustStructField {
                        name: snake_case(&name.value),
                        rust_type: if not_null {
                            sql2rstype(data_type)
                        } else {
                            format!("Option<{}>", sql2rstype(data_type))
                        },
                    })
                } else {
                    generated_tables
                        .get_mut(camelcase(ref_table.as_ref().unwrap().as_str()).as_str())
                        .expect(
                            format!(
                                "While generating {}: Expected to find an already generated table named `{}`.",
                                rs_struct.name,
                                ref_table.unwrap()
                            )
                            .as_str(),
                        )
                        .fields
                        .push(RustStructField {
                            name: snake_case(&ct.name.to_string()),
                            rust_type: format!("Vec<{}>", camelcase(&ct.name.to_string())),
                        });
                }
            }
        }
    }
    rs_struct
}

fn generate_nton_relation(ct: &ast::CreateTable) -> RustStruct {
    let mut rs_struct = RustStruct::new(camelcase(&ct.name.to_string()));
    for column in &ct.columns {
        let mut not_null = false;
        match column {
            ast::ColumnDef {
                name,
                data_type,
                options,
                ..
            } => {
                for option in options {
                    match option {
                        ast::ColumnOptionDef { option, .. } => match option {
                            ast::ColumnOption::NotNull => not_null = true,
                            ast::ColumnOption::ForeignKey { on_delete, .. } => {
                                if *on_delete == Some(ast::ReferentialAction::Cascade) {
                                    not_null = true;
                                }
                            }
                            _ => {}
                        },
                    }
                }
                rs_struct.fields.push(RustStructField {
                    name: snake_case(&name.value),
                    rust_type: if not_null {
                        sql2rstype(data_type)
                    } else {
                        format!("Option<{}>", sql2rstype(data_type))
                    },
                })
            }
        }
    }
    rs_struct
}

fn generate_output_file(opts: &InputOptions, parsed_ast: Vec<ast::Statement>) {
    let mut generated_structs = HashMap::new();
    for statement in parsed_ast {
        match statement {
            ast::Statement::CreateTable(ct) => {
                let db_table = generate_db_entity(&ct);
                let db_table_name = db_table.name.clone();
                generated_structs.insert(db_table_name, db_table);

                let table = match classify(&ct) {
                    Relationship::Entity => generate_entity(&ct),
                    Relationship::ListEntity(table) => {
                        generate_list_entity(&ct, &mut generated_structs)
                    }
                    Relationship::NToN => generate_nton_relation(&ct),
                };
                println!("Generated: {}", camelcase(table.name.clone().as_str()));
                let table_name = table.name.clone();
                generated_structs.insert(table_name, table);
            }
            ast::Statement::Insert(insert) => {
                println!("INSERT IS STILL TODO. This does do nothing");
            }
            _ => {}
        }
    }
    let mut file = std::fs::File::create(&opts.output).unwrap();
    for (_, table) in generated_structs {
        file.write_all(String::from(table).as_bytes()).unwrap();
    }
    file.flush().unwrap();
}
