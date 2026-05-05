// definition and some functions for a Table

use std::cmp::max;
use serde::{Deserialize, Serialize};

use crate::spec::*;


/// Represents a table extracted from the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// index into frags Vec of table start
    pub index: usize,
    /// table number as identified in the document
    pub number: u32,
    /// title/caption of the table
    pub title: String,
    /// page number where the table specification starts
    pub specpage: u32,
    /// type classification of the table
    pub tabtyp: TableType,
    /// horizontal x positions of columns
    pub colpos: Vec<i32>,
    /// names/headers of the columns
    pub colname: Vec<String>,
    /// row data, where each row is a vector of cells/strings
    pub rows: Vec<Vec<String>>,
    /// quirks counter for unhandled table formatting
    pub quirks: i32,
}


// key strings from JSON import/export, first level
pub const JSON_CREATED: &str       = "created";
pub const JSON_SPEC_FULLNAME: &str = "spec_fullname";
pub const JSON_SPEC_TYPE: &str     = "spec_type";
pub const JSON_TABLES: &str        = "tables";

// key strings from JSON import/export, per table
pub const JSON_NUMBER: &str    = "number";
pub const JSON_TITLE: &str     = "title";
pub const JSON_STARTPAGE: &str = "startpage";
pub const JSON_COLUMNS: &str   = "columns";
pub const JSON_QUIRKS: &str    = "quirks";
pub const JSON_ROWS: &str      = "rows";




pub fn debug_print_table_title(table: &Table) {
    println!("TABLE {:>06} {} Q{:>02} L{:>03} {:>3}  {}", table.index, table.tabtyp, table.quirks, table.rows.len(), table.number, table.title);
}


pub fn debug_print_table_pretty(table: &Table) {
    // debug_print_table_title(table);
    println!("[{}] {}", table.tabtyp, table.title);

    let mut col_lengths: Vec<usize> = vec![0; table.colname.len()];

    // column names
    for (i, name) in table.colname.iter().enumerate() {
        col_lengths[i] = max(col_lengths[i], name.len());
    }

    // all rows - calculate max line length for each column
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            // find the maximum line length within a cell (considering linebreaks)
            let max_line_len = cell.lines().map(|line| line.len()).max().unwrap_or(0);
            col_lengths[i] = max(col_lengths[i], max_line_len);
        }
    }

    // print column names
    let header: Vec<String> = table.colname.iter()
        .enumerate()
        .map(|(i, name)| {
            format!("{:<width$}", name, width = col_lengths[i])
        })
        .collect();
    println!("{}", header.join(" | "));

    // print separator line
    let separator: Vec<String> = col_lengths.iter()
        .map(|&len| "-".repeat(len))
        .collect();
    println!("{}", separator.join("-+-"));

    // print all rows with proper padding, also handling linebreaks
    for row in &table.rows {
        // split each cell into lines
        let cell_lines: Vec<Vec<&str>> = row.iter()
            .map(|cell| cell.lines().collect())
            .collect();

        // determine how many lines we need to print for this row
        let max_lines = cell_lines.iter()
            .map(|lines| lines.len())
            .max()
            .unwrap_or(0);

        // print line by line
        for line_idx in 0..max_lines {
            let formatted_row: Vec<String> = cell_lines.iter()
                .enumerate()
                .map(|(i, lines)| {
                    // Get the line at the current index, or empty string if this cell has fewer lines
                    let line = lines.get(line_idx).unwrap_or(&"");
                    format!("{:<width$}", line, width = col_lengths[i])
                })
            .collect();
        println!("{}", formatted_row.join(" | "));
    }
}
}

