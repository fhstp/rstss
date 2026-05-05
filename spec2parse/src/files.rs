// functions for filename(s) and file(s) handling, fetching the FODG files

use std::process::exit;
use regex::Regex;
use glob::glob;
use roxmltree::{Document};

use crate::helpers::parse_selection;


/// Search for all files matching the example passed in
///
/// # Arguments
/// * `input` - filename, for example "this/path/filename.1234.fodg"
///
/// # Returns
/// 3-part tuple containing:
/// 1. path part with trailing slash (or "./" for current directory)
/// 2. base name of file names (meaning without .1234.fodg part)
/// 3. Vec of matching filenames, alphabetically sorted, or empty Vec on failure
///
/// # Examples
/// ```
/// let (path, basename, files) = list_files(&"spec/foobar.1234.fodg");
/// ```
pub fn list_files(input: &str) -> (String, String, Vec<String>) {
    // let's find all files with a similar name (but differing page counter .xxxx.fodg)
    // Note: this function is today arguably more complex than needed,
    // but matching files via pattern was useful during early testing

    // split input into path part and filename part
    let (path_str, filename) = if let Some(pos) = input.rfind('/') {
        let path = &input[..pos];
        let filename = &input[pos+1..];
        (path.to_string(), filename.to_string())
    } else {
        (".".to_string(), input.to_string())
    };

    // ensure path has trailing slash "/"
    let path_with_slash = if path_str.is_empty() {
        "./".to_string()
    } else {
        let mut path = path_str.clone();
        if !path.ends_with('/') {
            path.push('/');
        }
        path
    };

    // split filename into parts
    let parts: Vec<&str> = filename.split('.').collect();
    let base_name = if parts.len() >= 3 {
        parts[..parts.len()-2].join(".")
    } else {
        String::new()
    };

    // should be something like basename.xxxx.fodg
    if parts.len() < 3 {
        return (path_with_slash, base_name, Vec::new());
    }

    let last_part = parts.last().unwrap();
    let second_last = parts[parts.len()-2];

    // check for .xxxx.fodg extension
    if *last_part != "fodg" || !second_last.chars().all(|c| c.is_ascii_digit()) {
        return (path_with_slash, base_name, Vec::new());
    }

    // now build regex pattern for matching files
    let escaped_base = regex::escape(&base_name);
    let pattern = format!("{}\\.\\d+\\.fodg$", escaped_base);
    let regex_filename = match Regex::new(&pattern) {
        Ok(regex) => regex,
        Err(_) => return (path_with_slash, base_name, Vec::new()),
    };


    let mut matching_files = Vec::new();

    // build path_str without trailing slash for use with glob below
    let glob_pattern = if path_str.is_empty() {
        "*".to_string()
    } else {
        let path_for_glob = if path_str.ends_with('/') {
            &path_str[..path_str.len()-1]        } else {
            &path_str
        };
        format!("{}/*", path_for_glob)
    };

    // enumerate files and find all matching files
    match glob(&glob_pattern) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(path) = entry {
                    if path.is_file() {  // is this really a file
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            if regex_filename.is_match(file_name) {
                                matching_files.push(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        Err(_) => {
            return (path_with_slash, base_name, Vec::new());
        }
    }

    matching_files.sort();
    (path_with_slash, base_name, matching_files)
}



/// Selects pages from the filenames vector based on a pages selection string.
///
/// Note: In the `filenames` vector usually index 0 corresponds to page 1, index 1 to page 2, and so on.
///
/// # Arguments
/// * `filenames` - A vector of all filenames of page files
/// * `pages` - A string describing the pages/files to select.
///   Examples:
///   - "all" - Select all pages
///   - "1" - Select only page 1
///   - "1,3-5,7" - Select page 1, pages 3 through 5, and page 7
///
/// # Returns
/// A vector containing only the filenames for the selected files/pages.
pub fn select_pages(filenames: Vec<String>, pages: &str) -> Vec<String> {
    // convenience case "all": return all filenames
    if pages.trim().eq_ignore_ascii_case("all") {
        return filenames;
    }

    let mut selected_pages = Vec::new();
    let page_indices = parse_selection(pages);

    for page_num in page_indices {
        if page_num > 0 && page_num <= filenames.len() {
            selected_pages.push(filenames[page_num - 1].clone());   // -1 offset as [0] is 0001.fodg
        }
    }

    selected_pages
}



/// Struct thats holds the data of a FODG (Flat Open Document Graphics) file
///
/// # Fields
/// * `filename` - the name of the FODG file (without path)
/// * `pagenum` - the page number from the filename (not as shown on a page)
/// * `filecontent` - the raw String content of the file
/// * `document` - the parsed XML document (from String) with a static lifetime
pub struct FODGfile {
    #[allow(dead_code)]
    pub filename: String,
    pub pagenum: u32,
    pub _filecontent: String,
    pub document: Document<'static>,  // document is a roxmltree::Document, a parsed XML
}


/// Read and parse multiple FODG files from the specified path.
///
/// # Arguments
/// * `path` - The base path to the directory containing the files
/// * `filenames` - A vector of filenames to read and parse
///
/// # Returns
/// A vector of `FODGfile` structs containing the filename, content, and parsed file as XML document.
///
/// # Errors
/// If a file cannot be read or parsed, the function will print an error message and exit immediately with a status code of 1.
pub fn read_fodg_files(path: &str, filenames: Vec<String>) -> Vec<FODGfile> {
    let mut fodg_docs = Vec::new();

    for filename in filenames {
        let full_path = format!("{}{}", path, filename);
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file {}: {}", full_path, e);
                exit(1);
            }
        };

        let doc = match Document::parse(&content) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error parsing XML file {}: {}", filename, e);
                exit(1);
            }
        };

        // Convert to 'static lifetime
        // This is safe as long as the content lives as long as the document.
        // This is a hack - but the assumption here is we have enough memory
        // for all input data and processing and free is the program ending.
        // (An alternative to the use of unsafe would be to leak a Box with the memory)
        let doc_static = unsafe {
            std::mem::transmute::<Document<'_>, Document<'static>>(doc)
        };

        // extract a page number from filename (which is not the number as display on the page!)
        // assume the filename is guaranteed to end with ".XXXX.fodg"
        let pagenum = {
            let parts: Vec<&str> = filename.rsplitn(3, '.').collect();
            parts[1].parse::<u32>().unwrap_or(0)
        };

        fodg_docs.push(FODGfile {
            filename,
            pagenum,
            _filecontent: content,
            document: doc_static,
        });
    }

    fodg_docs
}
