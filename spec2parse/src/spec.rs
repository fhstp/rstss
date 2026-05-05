// constants and code pieces related directly to the TPM specification

use std::process::exit;
use std::sync::LazyLock;
use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::{cmd_parse::Fragment, helpers::POLYGON_ID};


// specific strings that delimit subsections in tables of TPM spec part 3
pub const TABLEP3_HANDLES:    &str = "Handles";
pub const TABLEP3_PARAMETERS: &str = "Parameters";

// specific strings that delimit a multi-page table
pub const TABLECONT_NEXT: &str = "(continued on next page)";
pub const TABLECONT_PREV: &str = "(continued from previous page)";

// specific strings to search for, to find Table 1
pub const TABLE1_COLON: &str = "Table 1:";
// pub const TABLE1_SPACE: &str = "Table 1 ";

// and as regular expression
pub static RE_TABLE:  LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Table (\d{1,3}):").unwrap());


// Precompiled regular expressions
// for TPM spec part 2
pub static RE_DEF_CONST: LazyLock<Regex>    = LazyLock::new(|| Regex::new(r"Definition.+Constants").unwrap());
pub static RE_DEF_TYPEDEF: LazyLock<Regex>  = LazyLock::new(|| Regex::new(r"Definition of Types").unwrap());
pub static RE_DEF_ENUM: LazyLock<Regex>     = LazyLock::new(|| Regex::new(r"Definition.+Values").unwrap());
pub static RE_DEF_IFTYPE: LazyLock<Regex>   = LazyLock::new(|| Regex::new(r"Definition.+Type").unwrap());
pub static RE_DEF_STRUC: LazyLock<Regex>    = LazyLock::new(|| Regex::new(r"Definition.+Structure").unwrap());
pub static RE_DEF_UNION: LazyLock<Regex>    = LazyLock::new(|| Regex::new(r"Definition.+Union").unwrap());
pub static RE_DEF_BITFIELD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Definition.+Bits").unwrap());
// for TPM spec part 3
pub static RE_CMD_COMMAND: LazyLock<Regex>  = LazyLock::new(|| Regex::new(r"TPM2_.+ Command").unwrap());
pub static RE_CMD_RESPONSE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"TPM2_.+ Response").unwrap());


// Enum representing the different kinds of tables in TPM spec part 2 and 3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TableType {
    Const,
    Typedef,
    Enum,
    IfType,
    Struc,
    Union,
    Bitfield,
    Command,
    Response,
    Unknown    // or unimplemented (yet)
}

impl TableType {
    /// Determine the table type based on the table title string
    pub fn from_header(title: &str) -> Self {
        if        RE_DEF_CONST.is_match(title) {    TableType::Const
        } else if RE_DEF_TYPEDEF.is_match(title) {  TableType::Typedef
        } else if RE_DEF_ENUM.is_match(title) {     TableType::Enum
        } else if RE_DEF_IFTYPE.is_match(title) {   TableType::IfType
        } else if RE_DEF_STRUC.is_match(title) {    TableType::Struc
        } else if RE_DEF_UNION.is_match(title) {    TableType::Union
        } else if RE_DEF_BITFIELD.is_match(title) { TableType::Bitfield
        } else if RE_CMD_COMMAND.is_match(title) {  TableType::Command
        } else if RE_CMD_RESPONSE.is_match(title) { TableType::Response
        } else {
            TableType::Unknown  // or unknown/unimplemented
        }
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let short_name = match self {
            TableType::Const    => "Const",
            TableType::Typedef  => "Typde",
            TableType::Enum     => "Enum ",
            TableType::IfType   => "IfTyp",
            TableType::Struc    => "Struc",
            TableType::Union    => "Union",
            TableType::Bitfield => "Bitfi",
            TableType::Command  => "CMD  ",
            TableType::Response => "  RES",
            TableType::Unknown  => "?UNK?",
        };
        write!(f, "{}", short_name)
    }
}

impl std::fmt::Display for TableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}


pub const HEADER_MAX_Y: i32 = 9999;   // y-position of header must be smaller than this

// unique text fragments on first page of TPM 2.0 specification PDFs
// rev 185
pub const R185PART2:    &str = "Trusted Platform Module 2.0 Library Part 2:";
pub const R185PART3:    &str = "Trusted Platform Module 2.0 Library Part 3:";
// unique text fragments on first page of supplemental specification PDFs
pub const TCG_ALGORITHM_REGISTRY:     &str = "TCG Algorithm Registry";
pub const REGISTRY_OF_RESERVED:       &str = "Registry of Reserved TPM 2.0 Handles and Localities";
pub const TCG_TPM_VENDOR_ID_REGISTRY: &str = "TCG TPM Vendor ID Registry Family 1.2 and 2.0";


/// Enum representing different types of TPM specification documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecDocument {
    /// Part 2 of the TPM specification, which defines data structures
    Part2,
    /// Part 3 of the TPM specification, which defines commands
    Part3,
    /// TCG Algorithm Registry
    Algorithm,
    /// Registry of Reserved Handles and Localities
    Registry,
    /// TCG TPM Vendor ID Registry
    Vendors,
    /// Unknown, unrecognized or still unimplemented specification
    Unknown,
}

impl SpecDocument {
    /// Determine the table type based on a string
    pub fn from_str(thisstring: &str) -> Self {
        match thisstring {
            "Part2"     => SpecDocument::Part2,
        "Part3"     => SpecDocument::Part3,
        "Algorithm" => SpecDocument::Algorithm,
        "Registry"  => SpecDocument::Registry,
        "Vendors"   => SpecDocument::Vendors,
                  _ => SpecDocument::Unknown,
                }
    }
}


/// Parse detailed specification name from footer of page
pub fn parse_full_specname(frags: &Vec<Fragment>, input: usize) -> String {
    let mut specname= String::new();
    specname.push_str(&frags[input].text);
    let mut i = input;

    // repeat "while" as long as &frags[i].text does not contain a date-like string in format "mm/dd/yyyy" or "yyyy/mm/dd"
    while i < input+7 && !Regex::new(r"\d{1,4}\s*\/\s*\d{1,2}\s*\/\s*\d{1,4}").unwrap().is_match(&frags[i].text) {
        i += 1;
        specname.push_str(" ");
        specname.push_str(&frags[i].text);
    }
    specname = specname.replace("|", "");
    specname = specname.trim().to_string();
    specname = specname.split_whitespace().collect::<Vec<&str>>().join(" ");
    specname
}


/// Determine the type of specification document based on page 2's content
///
/// Checks for unique identifying strings that appear in the different specification documents.
///
pub fn identify_this_specification(frags: &Vec<Fragment>, startindex_page2: usize) -> (SpecDocument, String, i32, i32) {
    if frags.is_empty() { eprintln!("Error: empty document?"); exit(1); }

    let mut spec = SpecDocument::Unknown;
    let mut specname = String::new();
    let mut yheader = -1;
    let mut yfooter = -1;
    let mut i = startindex_page2;
    let thispage = frags[i].page;


    // loop over all fragments on this page
    while i<frags.len() && frags[i].page == thispage {
        let frag = &frags[i];

        // TPM spec itself and Algorithms spec only: have a footer on every page
        if        frag.text.contains(R185PART2) {
            spec = SpecDocument::Part2;     yfooter = frag.y;
            specname = parse_full_specname(frags, i);
        } else if frag.text.contains(R185PART3) {
            spec = SpecDocument::Part3;     yfooter = frag.y;
            specname = parse_full_specname(frags, i);
        } else if frag.text.contains(TCG_ALGORITHM_REGISTRY) {
            spec = SpecDocument::Algorithm; yfooter = frag.y;
            specname = parse_full_specname(frags, i);

            // Registry and Vendors spec: have header AND footer
        } else if frag.text.contains(REGISTRY_OF_RESERVED) {
            spec = SpecDocument::Registry;
            specname = parse_full_specname(frags, i);
            if frag.y < HEADER_MAX_Y { yheader = frag.y;
            } else { yfooter = frag.y; }
        } else if frag.text.contains(TCG_TPM_VENDOR_ID_REGISTRY) {
            spec = SpecDocument::Vendors;
            specname = parse_full_specname(frags, i);
            if frag.y < HEADER_MAX_Y { yheader = frag.y;
            } else { yfooter = frag.y; }
        }

        // we identified document and found footer position --> done
        if spec != SpecDocument::Unknown && yfooter != -1{
            // walk a bit back to set yfooter to start of box of footer (to skip ©)
            if frags[i-1].x <=0 && frags[i-1].style_id == POLYGON_ID { yfooter = frags[i-1].y; };  // registry, vendor
            if frags[i-2].x <=0 && frags[i-2].style_id == POLYGON_ID { yfooter = frags[i-2].y; };  // algo, part2 and part 3

            // debug_print_fragment_index(i, frag);
            break;
        }

        i +=1;
    }

    if spec == SpecDocument::Unknown {
        eprintln!("Help! Could not identify type of specification document :-("); exit(1);
    }

    (spec, specname, yheader, yfooter)
}


// constants for navigation in the TPM specification
// pub const TPM_MODULE_LIBRARY: &str = "Trusted Platform Module Library";
// pub const FAMILY20: &str = "Family “2.0”";
// pub const TCG_PUBLISHED: &str = "TCG Published";
