use std::collections::BTreeMap;
use std::process::exit;
use std::cmp::max;
use std::path::Path;

use regex::{Regex, Captures};
use serde::{Deserialize, Serialize};
use clap::Args;

use crate::files::*;
use crate::fonts::*;
use crate::helpers::*;
use crate::spec::*;
use crate::table::*;
use crate::xml::*;


#[derive(Debug, Args)]
#[command(help_template = "{about}\n{all-args}")]
pub struct Param {
    #[arg(long = "input", short = 'i', help="input sample file (one page, xxxx running number)", value_name="dir/foo.xxxx.fodg")]
    file_input: String,
    #[arg(long = "output", short = 'o', help="output parsed data to JSON file", value_name="filename.json")]
    file_output: Option<String>,
    #[arg(long = "overwrite", help="overwrite existing output file, if it already exists", default_value_t = false)]
    overwrite: bool,
    #[arg(long = "write-frag", help="write as parsed fragments to", value_name="filename")]
    filefrag: Option<String>,
    #[arg(long = "write-text", help="write as parsed lines of text to", value_name="filename")]
    filetext: Option<String>,
    #[arg(long = "list-tables", help="print list of tables in document", default_value_t = false)]
    listtables: bool,
    #[arg(long = "table", short = 't', help="select subset of table(s) to parse (default: all)", value_name="x,x-x,..")]
    tablenumber: Option<String>,
    #[arg(long = "pretty", help="pretty print parsed tables", default_value_t = false)]
    pretty: bool,
    #[arg(long = "page", short = 'p', help="select a subset of pages (default: all)", value_name="x,x-x,..")]
    pages: Option<String>,
    #[arg(long = "verbose", short = 'v', help="show some progress messages", default_value_t = false)]
    verbose: bool,
    #[arg(long = "quiet", short = 'q', help="suppress progress messages", default_value_t = true)]
    quiet: bool,
}



/// Parse FODG files (each file is a single page) and extract text fragments and styles.
///
/// # Arguments
///
/// * `fodg` - A vector of FODGfile structs (each containing the document content itself and page number)
///
/// # Returns
///
/// Returns a tuple containing:
/// * A vector of all Fragment structs found across all input files
/// * A BTreeMap mapping font IDs to font styles, representing all unique fonts found
pub fn parse_fodg_files(fodg: Vec<FODGfile>) -> (Vec<Fragment>, BTreeMap<u32, Font>) {
    let mut global_fonts: BTreeMap<u32, Font> = BTreeMap::new();
    let mut allfragments: Vec<Fragment> = Vec::new();

    for fodgfile in fodg {
        // println!("Filename: {}", fodgfile.filename);

        let node_root = xml_get_root_element(&fodgfile.document);
        check_measure_unit(&node_root);

        // first get map the fonts on this page as global unique ids on all pages
        let page_fonts = parse_font_styles(&node_root, &mut global_fonts);

        // extract all the text fragments and polygons on this page
        let pagefrags = extract_fragments(&node_root, fodgfile.pagenum, page_fonts);

        // concatenate page fragments to global Vec of fragments
        allfragments.extend(pagefrags);
    }

    (allfragments, global_fonts)
}


/// A text or polygon fragment of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragment {
    /// page number where this fragment appears
    pub page: u32,
    /// y-coordinate of the fragment's position
    pub y: i32,
    /// x-coordinate of the fragment's position
    pub x: i32,
    /// width of the fragment
    pub width: i32,
    /// height of the fragment
    pub height: i32,
    /// ID of the style applied to this fragment
    pub style_id: u32,
    /// text content of the fragment (custom text for a polygon fragment)
    pub text: String,
}


/// Find the starting index of the first fragment of a specified page.
///
/// # Arguments
/// * `frags` - A reference to a vector of Fragment objects to search through
/// * `page_to_find` - The page number to search for in the fragments
///
/// # Returns
/// The index of the first fragment belonging to the specified page.
pub fn frags_find_start_of_page(frags: &Vec<Fragment>, page_to_find: u32) -> usize {
    for i in 0..frags.len() {
        if frags[i].page == page_to_find { return i; }
    }
    println!("Error: searched for page {}, but it does not seem to exist?", page_to_find); exit(1);
}


/// Search for the next fragment matching a regular expression pattern.
///
/// # Arguments
/// * `frags` - A reference to a vector of Fragment objects to search through
/// * `start_index` - The index to start searching from (inclusive)
/// * `search_regex` - A reference to the Regex to match against fragment text
///
/// # Returns
/// Returns `Some((index, captures))` if a matching fragment is found:
/// * `index` - The index of the matching fragment
/// * `captures` - The regex captures for the match
///
/// Returns `None` if no matching fragment is found
pub fn frags_find_next_regex<'a>(frags: &'a Vec<Fragment>, start_index: usize, search_regex: &Regex) -> Option<(usize, Captures<'a>)> {
    for i in start_index..frags.len() {
        if let Some(m) = search_regex.captures(&frags[i].text) {
            return Some((i, m));
        }
    }
    return None;
}


/// Search for the first fragment whose `text` field contains the `search_for`
/// substring, starting from the specified `start_index`.
///
/// # Arguments
/// * `frags` - A reference to a vector of Fragment objects to search through
/// * `start_index` - The index in the vector to start searching from (inclusive)
/// * `search_for` - The substring to search for in the fragment's text field
///
/// # Returns
/// The index of the first fragment containing the searched for substring,
/// or `END_OF_DATA` constant if no matching fragment is found.
pub fn frags_find_next_text(frags: &Vec<Fragment>, start_index: usize, search_for: &str) -> usize {
    for i in start_index..frags.len() {
        if frags[i].text.contains(search_for) {
            return i;
        }
    }
    return END_OF_DATA;
}


/// Structure representing/holding the parsed document data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedData {
    /// kind of specification
    pub specdoc: SpecDocument,
    /// plain text name of the specification document
    pub specname: String,
    /// y-coordinate (cut-off) of each page header
    pub yheader: i32,
    /// y-coordinate (cut-off) of each page footer
    pub yfooter: i32,
    /// collection of tables parsed from the document
    pub tables: Vec<Table>
}

/// print one fragment's content
pub fn debug_print_fragment_index(index: usize, frag: &Fragment) {
   println!("i{:>06} P{:>03} Y{:>5} X{:>5}  W{:>5} H{:>4}  F{:>02}  {}", index, frag.page, frag.y, frag.x, frag.width, frag.height, frag.style_id, frag.text);
}

/// print multiple fragments for debugging (e.g. a text line)
pub fn debug_print_fragments_line_index(frags: &Vec<Fragment>, start_index: usize, last_index:usize) {
    for i in start_index..=last_index {
        print!("--p "); debug_print_fragment_index(i, &frags[i]);
    }
}

/// helper to assemble one string from multiple fragment's text content
pub fn assemble_string_from_fragments(frags: &[Fragment]) -> String {
    let mut assembled_text = String::new();
    for frag in frags {
        assembled_text.push_str(&frag.text);
    }
    assembled_text
}



/// helper function to "best guess" the start of the next line of text
pub fn frag_find_start_next_line(frags: &Vec<Fragment>, start_index: usize) -> usize {
    let mut i = start_index;
//    while frags[i].style_id == POLYGON_ID { i +=1 }  // ignore polygons

    // y position of start fragment
    let ystart = frags[i].y;

    // y position of next line,
    // guess the minimum y distance of the next line from the current height
    let ynext  = ystart + (frags[i].height/10*9);  // add ~90% height of current fragment

    loop {
        i +=1;
        if frags[i].y < ystart { break }   // next page --> is surely a next line
        if frags[i].y < ynext { continue } // still in same line
        break  // have found next line
    }

    // println!("  now {} > {} guessed", frags[i].y-ystart, ynext-ystart);
    i
}


/// Find all the tables in the document
/// (=find all starting indexes of the respective table start fragments)
pub fn find_tables(frags: &Vec<Fragment>, start_index: usize, data: &mut ParsedData) -> () {
    let mut i = start_index;
    let _style;

    // first, we want to find Table 1
    // unfortunately, tables and table of contents look not the same for every document
    match data.specdoc {
        SpecDocument::Part2 | SpecDocument::Part3 | SpecDocument::Algorithm | SpecDocument::Registry => {
            i = frags_find_next_text(&frags, i, TABLE1_COLON);
            if data.specdoc == SpecDocument::Registry {
                // finds TOC first, so continue to find next match
                i = frags_find_next_text(&frags, i+1, TABLE1_COLON);
            }
            _style = frags[i].style_id;
        },

        // TODO
        // SpecDocument::Vendors => {
        //     i = frags_find_next_text(&frags, i, TABLE1_SPACE);
        //     debug_print_fragment_index(i, &frags[i]);
        //     println!("vendor spec not implemented yet due to inconsistencies - stopped"); exit(1);
        // },

        _ => { println!("Don't know how to find tables in this specification - stopped"); exit(1); },
    }

    // ok, found Table 1 fragment so far, so start parse from there
    while i < frags.len() {
        // find next table header by regex text search
        let capture = match frags_find_next_regex(&frags, i, &RE_TABLE) {
            Some((new_i, cap)) => {
                i = new_i;
                cap
            },
            None => break,  // no more Table found by regex
        };

        if i == END_OF_DATA { break; }  // no more Table found - because end of text

        let mut title = frags[i].text.clone();  // "Table x:" fragment"
        title.push_str(&frags[i+1].text);       // rest of table title

        let tabtyp = TableType::from_header(&title);   // identify what kind of table

        let table = Table {
            index: i,
            number: capture.get(1).unwrap().as_str().parse().unwrap(),
            title,
            specpage: frags[i].page,
            tabtyp,
            colpos: Vec::new(),
            colname: Vec::new(),
            rows: Vec::new(),
            quirks: 0
        };
//        debug_print_table(&table);
//        debug_print_fragment_index(i, &frags[i]);

        data.tables.push(table);

        i += 1;
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum ParseState {
    TableHeader,
    ReadRow,
    Done,
}


/// Best guess check if this is horizontal line of a table
///
/// This function verifies that the fragment at index `i` represents a horizontal line by:
/// 1. Checking that there is exactly one fragment between `nexti` and `i`
/// 2. Verifying that the fragment is a polygon (style_id equals POLYGON_ID)
/// 3. Ensuring the fragment has zero height (which is characteristic of a horizontal line)
///
/// If any of these conditions are not met, the function prints an error message with
/// details about the fragments and exits the program.
pub fn check_for_horizontal_line(frags: &Vec<Fragment>, nexti:usize, i: usize, message: &str) -> () {
    if (nexti-i) != 1 || frags[i].style_id != POLYGON_ID || frags[i].height != 0 {
        println!("\n{}   FRAGS{} STY{} H{} ", message, nexti-i, frags[i].style_id, frags[i].height);
        debug_print_fragments_line_index(frags, i, nexti);
        exit(1);
    };
}


/// Cursed main parser to extract the information from a table in the specification
///
/// Note: this function is still unfinished and cannot fully parse all tables
/// (maybe a from-scratch rewrite with all the learnings would be efficient?)
///
/// # Arguments
/// * `frags` - A reference to a vector of Fragment objects
/// * `data` - the ParsedData struct containing global document information
/// * `table_number` - the table to parse
///
/// # Returns
/// This function does not return a value. It modifies the `table` and `data` arguments
/// in place to populate them with parsed table data.
pub fn parse_one_table(frags: &Vec<Fragment>, data: &mut ParsedData, table_number: usize) -> () {
    const DEBUG: bool = false;

    let table = &mut data.tables[table_number];   // -1 because Vec starts [0] but Tables are numbered from 1
    let mut i = table.index;  // current postion
    let mut nexti;            // next interesting position (e.g. start of next line)

    if DEBUG {
        debug_print_table_title(table);
        println!("yhead:{} yoff:{}", data.yheader, data.yfooter);
    }

    nexti = frag_find_start_next_line(&frags, i);
    table.title = assemble_string_from_fragments(&frags[i..nexti]);   // title of table
    i = nexti;

    let mut parsestate = ParseState::TableHeader;
    let mut linecount = 0;


    loop {
        if DEBUG { println!(""); }
        if parsestate == ParseState::Done { break; }

        if parsestate == ParseState::TableHeader {
            nexti = frag_find_start_next_line(&frags, i);
            if DEBUG { debug_print_fragment_index(i, &frags[i]); }

            // === horizontal line of table start expected
            check_for_horizontal_line(frags, nexti, i, &"parse_table: line after table title not found! abort!");
            i = nexti;

            // next part should be the table header line with the columns and their names
            nexti = frag_find_start_next_line(&frags, i);
            if DEBUG { debug_print_fragments_line_index(&frags, i, nexti-1); }

            let cols = table.colpos.len();
            if cols>0 { // we parsed the header already on a previous page
                    if DEBUG { println!("  AGAIN  {}", cols); }
                // check that this header is same
                for j in 0..cols {
                    if DEBUG { println!("i{} j{}  x{:>05} colposx{:>05}",i,j,frags[i+j].x, table.colpos[j]); }
                    if frags[i+j].style_id != POLYGON_ID ||
                       frags[i+j].width != 0 ||
                       frags[i+j].x != table.colpos[j] {
                        println!("parse_table: table header on new page does not match previous page?");
                        debug_print_fragment_index(i+j, &frags[i+j]);
                        exit(1);
                    }
                }

            } else { // first page of a table, we need to parse header

                // first find x positions of vertical lines
                let mut loopi = i;
                while loopi < nexti {
                    if frags[loopi].style_id == POLYGON_ID && frags[loopi].width == 0 {
                        table.colpos.push(frags[loopi].x);
                    } else {
                        break;
                    }
                    loopi += 1;
                }

                table.colname.resize(table.colpos.len()-1, String::new());  // allocate column names

                // then find all column names
                while loopi < nexti {
                    // find the appropriate column index based on fragment's x position
                    let mut col_index = 0;
                    while frags[loopi].x >= table.colpos[col_index+1] {
                        col_index += 1;
                    }

                    // add the text to the appropriate column name
                    if col_index < table.colname.len() {
                        table.colname[col_index].push_str(&frags[loopi].text);
                    }
                    loopi += 1;
                }

                // sanity check
                if table.colpos.len()-1 != table.colname.len() {
                    debug_print_fragment_index(nexti, &frags[nexti]);
                    println!("Columns positions: {:?}", table.colpos);
                    println!("Columns names: {:?}", table.colname);
                    println!("parse_table: got {} columns but {} names (should be one less!)",
                        table.colpos.len(), table.colname.len());
                    exit(1);
                }

                if DEBUG {
                    println!("===> column positions: {:?}", table.colpos);
                    println!("===> column names: {:?}", table.colname);
                }
            }

            i = nexti;

            // horizontal line after table header
            parsestate = ParseState::ReadRow;
            continue;

        } // TableHeader


        if parsestate == ParseState::ReadRow {
            nexti = frag_find_start_next_line(&frags, i);
            if nexti-i>1 && frags[i].style_id == POLYGON_ID && frags[i].width == 0 && frags[i].height>0 && table.rows.len()==0 {
                // table with vertically merged cells - we can't parse this -> abort
                table.quirks += 1;
                parsestate = ParseState::Done;
                continue;
            }
            check_for_horizontal_line(frags, nexti, i, &"parse_row: horizontal line expected, but not found? abort!");
            if DEBUG {
                debug_print_fragment_index(i, &frags[i]);
                println!("");
            }
            i = nexti;

            // nothing after horizontal line on this page -> table must have ended and rest of page is empty
            if frags[i].y >= data.yfooter {
                parsestate = ParseState::Done;
                continue;
            }

            // ignore decorative boxes in text
            while frags[i].style_id == POLYGON_ID &&
                  frags[i].width>0 &&
                  frags[i].height>0 { i+=1; }

            // there is text after the horizontal line
            if frags[i].style_id != POLYGON_ID {
                // is it a table that continues on next page?
                if frags[i].text == TABLECONT_NEXT {
                    if DEBUG {
                        println!("...NEXT..................................");
                        debug_print_fragment_index(i, &frags[i]);
                    }

                    // read until end of page
                    while frags[i].page <= frags[nexti].page { i += 1; }

                    if DEBUG {
                        println!("...PAGE..................................");
                        debug_print_fragment_index(i, &frags[i]);
                    }
                    // read until continue text at start of table
                    while frags[i].text != TABLECONT_PREV { i += 1; }

                    if DEBUG {
                        println!("...PREV..................................");
                    }

                    // we are on next page and expect the table header again
                    i+=1;
                    parsestate = ParseState::TableHeader;
                    continue;
                }

                // if after a horizontal line there is any text, the table is finished
                parsestate = ParseState::Done;
                continue;

            } else {
                // check for the vertical lines
                let mut found = 0;
                while frags[i+found].style_id == POLYGON_ID && frags[i+found].width == 0 {
                    // debug_print_fragment_index(i+found, &frags[i+found]);
                    found +=1;
                }

                if found != table.colpos.len() {
                    if found == 0 {
                        println!("read_row: mismatch in columns, expected {} got {}", table.colpos.len(), found);
                        debug_print_fragment_index(i, &frags[i]);
                        exit(1)
                    }

                    // println!("");
                    // debug_print_fragment_index(i, &frags[i]);

                    // there are most likely horizontally merged cells,
                    // we can't parse this, so skip this table row
                    table.quirks += 1;
                    let yskip = frags[i].y + frags[i].height;
                    while frags[i].y < yskip {
                        i +=1;
                        // debug_print_fragment_index(i, &frags[i]);

                        // for TPM spec part 3, detection of section separators -> add custom table row
                        if frags[i].text == TABLEP3_HANDLES {
                            let mut cells= vec![String::new(); table.colname.len()];
                            cells[0].push_str("-- ");
                            cells[0].push_str(TABLEP3_HANDLES);
                            cells[0].push_str(" --");
                            table.rows.push(cells);
                        } else if frags[i].text == TABLEP3_PARAMETERS {
                            let mut cells= vec![String::new(); table.colname.len()];
                            cells[0].push_str("-- ");
                            cells[0].push_str(TABLEP3_PARAMETERS);
                            cells[0].push_str(" --");
                            table.rows.push(cells);
                        }
                    }
                    continue
                }

                // compute height of one table row from first vertical line
                let row_ymax = frags[i].y + frags[i].height;
                i = i + found; // skip over the polygon fragments

                let mut line_ymax;  // max height of a line of text

                // temporary storage for text in this row
                let mut cells= vec![String::new(); table.colname.len()];

                // loop over individual text lines in a table row
                loop {
                    // ignore decorative background boxes
                    while frags[i].style_id == POLYGON_ID && frags[i].width>0 && frags[i].height>0 { i +=1; }

                    // table with vertical merged cells start a line with vertical lines
                    // not implemented -> abort
                    if frags[i].style_id == POLYGON_ID && frags[i].width == 0 {
                        table.quirks += 1;
                        if DEBUG {
                            println!("MERGE abort");
                        }
                        parsestate = ParseState::Done;
                        break;
                    }

                    line_ymax = frags[i].y + frags[i].height;
                    if DEBUG {
                        println!("  lineymax:{}", line_ymax);
                    }

                    while frags[i].y < line_ymax {
                        if DEBUG {
                            debug_print_fragment_index(i, &frags[i]);
                        }
                        line_ymax = max(line_ymax, frags[i].y + frags[i].height);  // redo, one line may have different fonts

                        // find the column index based on fragment's x position
                        let mut col_index = 0;
                        while frags[i].x > table.colpos[col_index] {
                            col_index += 1;
                        }
                        col_index -= 1;

                        cells[col_index].push_str(&frags[i].text);
                        if DEBUG {
                            println!("i{:>06} col{}  {}", i, col_index, &frags[i].text);
                        }
                        i+=1;

                        // alternative, detect new line by x-coordinate
                        if frags[i].x < frags[i-1].x {
                            break
                        }
                    }

                    // check if this table row is done
                    if frags[i].y > row_ymax && frags[i].style_id == POLYGON_ID {
                        break

                    } else if frags[i].y > row_ymax && frags[i].style_id != POLYGON_ID{
                        println!("parse_row: BUG? polygon expected?");
                        debug_print_fragment_index(i, &frags[i]);
                        exit(1);
                    }

                    // since we have not exited the loop,
                    // there should be another line of text in this table row,
                    // therefore add \n to each cell
                    for index in 0..table.colname.len() {
                        if !cells[index].is_empty() && !cells[index].ends_with('\n') {
                            cells[index].push_str("\n");
                        }
                    }
                }

                // remove any trailing \n in cells
                for cell in cells.iter_mut() {
                    *cell = cell.trim_end_matches('\n').to_string();
                }
                table.rows.push(cells);

            }

        } // ReadRow

        linecount += 1;
        if linecount == 10000 {break};

    } // loop over parser states

    if DEBUG { println!(""); }
}


/// This function receives the fragments as input and parses/extracts all useful data/tables
pub fn extract_data_from_fragments(frags: Vec<Fragment>, quiet: bool, tablenumber: &Option<String>) -> ParsedData {
    // look at page 2...
    let i = frags_find_start_of_page(&frags, 2);
    // ...to identify the type of specification document, and determine header and footer cutoff/position
    let (specdoc, specname ,yheader, yfooter) = identify_this_specification(&frags, i);
    if !quiet { println!("Found {:?}", specdoc); }

    // data structure for all accumulated knowledge
    let mut data =
        ParsedData {
            specdoc,
            specname,
            yheader: yheader,
            yfooter: yfooter,
            tables: Vec::new(),
        };

    // find positions of all the tables
    find_tables(&frags, i, &mut data);
    if !quiet { println!("Tables found: {}", data.tables.len()); }


    // and parse one or more tables
    if let Some(thisstring) = tablenumber {
        let tables_vec: Vec<usize> = parse_selection(&thisstring);

        for table_to_parse in tables_vec {
            // find the table matching the requested table number
            let table_index = match data.tables.iter().position(|t| t.number == table_to_parse as u32) {
                Some(idx) => idx,
                None => { println!("Help? Requested table number {} not found - stop", table_to_parse); exit(1); }
            };
            parse_one_table(&frags, &mut data, table_index);
        }

    // if no table(s) specified, just parse all tables
    } else {
        for table_index in 0..data.tables.len() {
            parse_one_table(&frags, &mut data, table_index);
        }
    }

    data
}


/// export data structures/knowledge gained to a file
pub fn export_data_to_json(filename: &str, data: ParsedData, quiet: bool) -> () {
    use serde_json::json;
    use chrono::Local;
    use std::io::BufWriter;

    let file = match std::fs::File::create(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: Could not create file '{}': {}", filename, e);
            exit(1);
        }
    };

    // build the JSON structure straight via the json! macro
    // this removes the need for manual string formatting and escaping,
    // which serde_json handles automatically and correctly
    let tables_json: Vec<serde_json::Value> = data.tables.iter().map(|table| {
        json!({
            JSON_NUMBER: table.number,
            JSON_TITLE: table.title,
            JSON_STARTPAGE: table.specpage,
            JSON_COLUMNS: table.colname,
            JSON_QUIRKS: table.quirks,
            JSON_ROWS: table.rows
        })
    }).collect();

    let output = json!({
        JSON_SPEC_FULLNAME: data.specname,
        JSON_SPEC_TYPE: data.specdoc,
        JSON_CREATED: Local::now().format("%Y%m%d-%H%M%S").to_string(),
        JSON_TABLES: tables_json
    });

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &output).unwrap();
    if !quiet {
        println!("Data exported successfully to '{}'.", filename, );
    }
}



// main function

pub fn run(param: Param) -> () {
    let mut quiet = param.quiet;
    if param.verbose { quiet = false; }

    // determine candidates of input files, from passed input filename
    let (path, basename, mut filenames) = list_files(&param.file_input);

    // select a subset of files/pages, if requested (for faster runs)
    if param.pages.is_some() {
        filenames = select_pages(filenames, &param.pages.unwrap());
    }

    if filenames.is_empty() {
        println!("Error: no pages found, no work to be done?"); exit(1);
    } else if !quiet {
        let first = &filenames[0];
        let last = filenames.last().unwrap();
        println!("Reading from {} {}  ({} ... {})  {} file(s)", path, basename, first, last, filenames.len());
    }

    // read/parse all FODG files into memory
    let fodg_docs = read_fodg_files(&path, filenames);

    // extract from the FODG XML all of the interesting data fragments
    let (frags, _global_fonts) = parse_fodg_files(fodg_docs);
    if !quiet { println!("Fragments extracted: {}", frags.len()); }

    // for debugging, output the parsed fragments or as text
    if param.filefrag.is_some() {
        debug_dump_fragments_to_file(&param.filefrag.unwrap(), &frags, false, quiet); }
    if param.filetext.is_some() {
        debug_dump_fragments_to_file(&param.filetext.unwrap(), &frags, true, quiet); }

    // extract all structured (tables) data from the fragments
    let data = extract_data_from_fragments(frags, quiet, &param.tablenumber);


    // ok parsing complete, now output/print in different ways


    // list only the tables found
    if param.listtables {
        for table_index in 0..data.tables.len() {
            debug_print_table_title(&data.tables[table_index]);
        }
    }

    // pretty print parsed tables
    if param.pretty {
        for table_index in 0..data.tables.len() {
            let table = &data.tables[table_index];
            if table.rows.len()>0 {
                println!("\n");
                println!("// Extracted from {}, page {}", data.specname, table.specpage);
                debug_print_table_pretty(&data.tables[table_index]);
            }
        }
    }

    // export to JSON
    if let Some(fileexport) = param.file_output {
        if Path::new(&fileexport).exists() {
            if param.overwrite {
                if std::fs::remove_file(&fileexport).is_err() {
                    eprintln!("Error: Could not remove file '{}' to overwrite?", fileexport); exit(1);
                }
            } else {
                eprintln!("File '{}' already exists. Use --overwrite to replace it.", fileexport); exit(1);
            }
        }
        export_data_to_json(&fileexport, data, quiet);
    }

}
