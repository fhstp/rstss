use std::process::exit;
use std::path::Path;

use serde::{Deserialize, Serialize};
use clap::Args;

use crate::helpers::parse_selection;
use crate::spec::*;
use crate::table::*;

#[derive(Debug, Args)]
#[command(help_template = "{about}\n{all-args}")]
pub struct Param {
    #[arg(long = "input", short = 'i', help="read parsed data from", value_name="filename")]
    file_input: String,
    #[arg(long = "table", short = 't', help="select subset of table(s) to generate (default: all)", value_name="x,x-x,..")]
    tablenumber: Option<String>,
    #[arg(long = "verbose", short = 'v', help="show some progress messages", default_value_t = false)]
    verbose: bool,
    #[arg(long = "quiet", short = 'q', help="suppress progress messages", default_value_t = true)]
    quiet: bool,
    #[arg(long = "rust", short = 'r', help="generate Rust code skeleton", default_value_t = false)]
    genrust: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputData {
    pub specdoc: SpecDocument,
    pub specname: String,
    pub tables: Vec<Table>
}

/// import data structures from a JSON file
pub fn import_data(filename: &str, quiet: bool) -> InputData {
    use std::io::BufReader;
    use std::fs::File;
    use serde_json::Value;

    let file = File::open(filename).expect(&format!("Error: Failed open of file: {}", filename));
    let reader = BufReader::new(file);

    let json_data: Value = serde_json::from_reader(reader).expect(&format!("Error: Failed to parse JSON from file: {}", filename));

    // the top-level informational fields
    let specname = json_data[JSON_SPEC_FULLNAME].as_str().unwrap_or("").to_string();
    let specdoc: SpecDocument = SpecDocument::from_str(json_data[JSON_SPEC_TYPE].as_str().unwrap());
    // println!("Specification Name: {}", specname);
    // println!("Specification Type: {:?}", specdoc);

    // and the tables itself
    let tables: Vec<Table> = match json_data[JSON_TABLES].as_array() {
        Some(t) => t.iter().map(|table_json| {
            Table {
                number: table_json[JSON_NUMBER].as_u64().unwrap() as u32,
                title: table_json[JSON_TITLE].as_str().unwrap().to_string(),
                specpage: table_json[JSON_STARTPAGE].as_u64().unwrap() as u32,
                colname: table_json[JSON_COLUMNS].as_array().unwrap().iter().map(|c| c.as_str().unwrap().to_string()).collect(),
                rows: table_json[JSON_ROWS].as_array().unwrap().iter().map(|row| {
                    row.as_array().unwrap().iter().map(|cell| cell.as_str().unwrap().to_string()).collect()
                }).collect(),
                quirks: table_json[JSON_QUIRKS].as_u64().unwrap() as i32,
                // recompute type from header
                tabtyp: TableType::from_header(&table_json[JSON_TITLE].as_str().unwrap().to_string()),
                // not in data, fake data
                colpos: Vec::new(),
                index: 0,
            }
        }).collect(),
        None => Vec::new(),
    };

    if !quiet {
        println!("Data imported from '{}'", filename);
    }

    let data =
        InputData {
            specdoc,
            specname,
            tables: tables
        };

    data
}


fn generate_const_table(data: &InputData, table: &Table) {
    println!("\n");
    println!("// Extracted from {}, page {}", data.specname, table.specpage);

    println!("TODO");
}



pub fn generate_rust(data: &InputData, table: &Table) {
    match table.tabtyp {
        TableType::Const => generate_const_table(data, table),
        TableType::Typedef => println!("// Typedef tables not yet supported"),
        TableType::Enum => println!("// Enum tables not yet supported"),
        TableType::IfType => println!("// IfType tables not yet supported"),
        TableType::Struc => println!("// Structure tables not yet supported"),
        TableType::Union => println!("// Union tables not yet supported"),
        TableType::Bitfield => println!("// Bitfield tables not yet supported"),
        TableType::Command => println!("// Command tables not yet supported"),
        TableType::Response => println!("// Response tables not yet supported"),
        _ => { println!("// Unhandled table type: {} Help?", table.title); exit(1) },
    }
}


pub fn do_one_table(data: &InputData, table: &Table, genrust: bool) {
    if genrust && table.tabtyp != TableType::Unknown {
        generate_rust(data, table);
    } else {
        if table.tabtyp == TableType::Unknown {
            println!("// Unknown table type, cannot generate code for this")
        }
        println!("\n");
        println!("// Extracted from {}, page {}", data.specname, table.specpage);
        debug_print_table_pretty(table);
    }
}


pub fn run(param: Param) -> () {
    let mut quiet = param.quiet;
    if param.verbose { quiet = false; }

    if !Path::new(&param.file_input).exists() {
        eprintln!("Error: input file '{}' does not exist?", param.file_input); exit(1);
    }


    let data = import_data(&param.file_input, quiet);


    if let Some(thisstring) = param.tablenumber {
        let tables_vec: Vec<usize> = parse_selection(&thisstring);

        for table_to_parse in tables_vec {
            // find the table matching the requested table number
            let table_index = match data.tables.iter().position(|t| t.number == table_to_parse as u32) {
                Some(idx) => idx,
                None => { println!("Help? Requested table number {} not found - stop", table_to_parse); exit(1); }
            };
            do_one_table(&data, &data.tables[table_index], param.genrust);
        }

    } else { // if no table(s) specified, all tables
        for table_index in 0..data.tables.len() {
            // print_one_table_raw(&data.tables[table_index]);
            do_one_table(&data, &data.tables[table_index], param.genrust);
        }
    }

}
