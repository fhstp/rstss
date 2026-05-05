// function for handling the XML from FODG files with roxmltree

use std::collections::BTreeMap;
use std::process::exit;
use std::fs::File;
use std::io::Write;

use roxmltree::{Document, Node};

use crate::helpers::*;
use crate::cmd_parse::Fragment;


/// Retrieve the root element for an XML document
///
/// # Arguments
/// * `document` - A reference to a `roxmltree::Document` representing the parsed XML document.
///
/// # Returns
/// Returns a `roxmltree::Node` representing the root element of the XML document.
///
pub fn xml_get_root_element<'a>(document: &'a Document) -> Node<'a,'a> {
    return document.root_element();  // document is a roxmltree::Document
}


/// Find all matching sibling nodes by tag name and optional attribute
///
/// # Returns
/// A vector of all matching sibling nodes. Returns an empty vector if no matches are found.
pub fn xml_find_all_children_by_name<'a, 'b>(node: Node<'a, 'b>, name: &str, attribute: Option<(&str, &str)>)
-> Vec<Node<'a, 'b>> {
    let mut matching_nodes = Vec::new();

    let mut current_node = match node.first_element_child() {
        Some(child) => child,
        None => return matching_nodes,
    };

    loop {
        if current_node.tag_name().name() == name {
            // If tag name matches, also check for attribute (if provided)
            let attribute_matches = match attribute {
                Some((attr_name, attr_value)) => {
                    // Check if the attribute exists and has the expected value
                    current_node
                        .attribute(attr_name)
                        .map_or(false, |found_attr| found_attr == attr_value)
                }
                None => true, // No attribute provided, so it's a match
            };

            if attribute_matches {
                matching_nodes.push(current_node);
            }
        }

        // Move to the next sibling
        match current_node.next_sibling_element() {
            Some(next_node) => current_node = next_node,
            None => break,
        }
    }

    matching_nodes
}



/// Find the first matching sibling node by tag name and optional attribute.
///
/// # Returns
/// The first matching sibling node. Aborts with an error if no match is found.
pub fn xml_find_onechild_by_name<'a, 'b>(node: Node<'a, 'b>, name: &str, attribute: Option<(&str, &str)>)
-> Node<'a, 'b> {
    let matching_nodes = xml_find_all_children_by_name(node, name, attribute);

    if let Some(first_match) = matching_nodes.into_iter().next() {
        first_match
    } else {
        // No match found, abort with error
        let pos = node.document().text_pos_at(node.range().start);
        eprintln!("Error: Could not find an element with tag name '{}'{}",name,
            match attribute {
                Some((attr_name, attr_value)) => {
                    format!(" and attribute {}='{}'", attr_name, attr_value)
                }
                None => String::new(),
            }
        );
        eprintln!("Starting node was: '{}' at L{}:C{}", node.tag_name().name(), pos.row, pos.col);
        exit(1);
    }
}


/// Validate the MeasureUnit configuration in an XML document.
///
/// # Arguments
/// * `node_root` - A reference to the root node of the XML document.
pub fn check_measure_unit(node_root: &roxmltree::Node) -> () {
    let node_settings = xml_find_onechild_by_name(*node_root,
        "settings", None);
    let node_item_set = xml_find_onechild_by_name(node_settings,
        "config-item-set", Some(("name","ooo:configuration-settings")));
    let node_measure_unit = xml_find_onechild_by_name(node_item_set,
        "config-item", Some(("name","MeasureUnit")));

    let measure_unit_value = node_measure_unit.text().unwrap_or("unknown").trim();
    if measure_unit_value != "7" {
        eprintln!("Error: expected MeasureUnit '7', but found '{}' instead?", measure_unit_value);
        exit(1);
    }
}


/// Helper function to parse a size string, remove "cm", multiply by 1000, and convert to i32
fn parse_and_convert_cm(s: &str) -> i32 {
    (s.trim_end_matches("cm")
        .parse::<f64>()
        .unwrap_or(0.0) * 1000.0) as i32
}

/// Extract all polygons - the pieces that make up the cell borders of tables and save them as fragment
pub fn extract_polygons(node_page: &roxmltree::Node, pagenumber: u32) -> Vec<Fragment> {
    let mut node_item_set = xml_find_all_children_by_name(*node_page,
        "polygon", Some(("layer","layout")));
    // println!(" polygons found: {}", node_item_set.len());

    let node_item_set_lines = xml_find_all_children_by_name(*node_page,
    "line", Some(("layer","layout")));
    // println!(" lines found: {}", node_item_set_lines.len());
    node_item_set.extend(node_item_set_lines);


    let mut frags: Vec<Fragment> = Vec::new();

    // loop over all polygons/lines
    for node in node_item_set {
        let width;
        let height;
        let x;
        let y;
        let style      = node.attribute("text-style-name").unwrap_or_else(|| {      eprintln!("Error: expected 'text-style-name' attribute"); exit(1); });

        if node.tag_name().name()=="polygon" {
            let width_str  = node.attribute("width").unwrap_or_else(|| {  eprintln!("Error: expected 'width' attribute"); exit(1); });
            let height_str = node.attribute("height").unwrap_or_else(|| { eprintln!("Error: expected 'height' attribute"); exit(1); });
            let x_str      = node.attribute("x").unwrap_or_else(|| {      eprintln!("Error: expected 'x' attribute"); exit(1); });
            let y_str      = node.attribute("y").unwrap_or_else(|| {      eprintln!("Error: expected 'y' attribute"); exit(1); });
            width  = parse_and_convert_cm(width_str) as i32;
            height = parse_and_convert_cm(height_str) as i32;
            x      = parse_and_convert_cm(x_str) as i32;
            y      = parse_and_convert_cm(y_str) as i32;
        } else {  // line
            let x1_str      = node.attribute("x1").unwrap_or_else(|| {      eprintln!("Error: expected 'x1' attribute"); exit(1); });
            let x2_str      = node.attribute("x2").unwrap_or_else(|| {      eprintln!("Error: expected 'x2' attribute"); exit(1); });
            let y1_str      = node.attribute("y1").unwrap_or_else(|| {      eprintln!("Error: expected 'y1' attribute"); exit(1); });
            let y2_str      = node.attribute("y2").unwrap_or_else(|| {      eprintln!("Error: expected 'y2' attribute"); exit(1); });
            let x1  = parse_and_convert_cm(x1_str) as i32;
            let x2  = parse_and_convert_cm(x2_str) as i32;
            let y1  = parse_and_convert_cm(y1_str) as i32;
            let y2  = parse_and_convert_cm(y2_str) as i32;

            x = x1.min(x2);
            y = y1.min(y2);
            width = x1.abs_diff(x2) as i32;
            height = y1.abs_diff(y2) as i32;
        }

        // add
        let poly = if (width - height).abs() <= 2 {
            Fragment { page: pagenumber, y, x, width, height, style_id: POLYGON_ID, text: SQUARE.to_string()+style }
        } else if width > height {
            Fragment { page: pagenumber, y, x, width, height, style_id: POLYGON_ID, text: HORI.to_string()+style }
        } else { // width < height
            Fragment { page: pagenumber, y, x, width, height, style_id: POLYGON_ID, text: VERT.to_string()+style }
        };

        if frags.len()>0 {
            let lastfrag = frags.last().unwrap();
            if lastfrag.height==poly.height &&
                lastfrag.width==poly.width &&
                lastfrag.x==poly.x &&
                lastfrag.y==poly.y {
                // don't add, it's a duplicate
            } else {
                frags.push(poly);
            }
        } else {
            frags.push(poly);
        }

    }

    frags.sort_by(|a, b| {
        a.y.cmp(&b.y)
            .then_with(|| a.x.cmp(&b.x))
    });

    // for frag in &frags {
    //     println!("P{:>03} Y{:>5} X{:>5}  W{:>5} H{:>4}  F{:>02}  {}", frag.page, frag.y, frag.x, frag.width, frag.height, frag.style_id, frag.text);
    // }

    frags

}


/// Extract all text and polygon fragments - these are the pieces that carry interesting information for us - on this page
pub fn extract_fragments(node_root: &roxmltree::Node, pagenumber: u32, fontmapping: BTreeMap<String, u32>) -> Vec<Fragment> {
    let node_body = xml_find_onechild_by_name(*node_root,
        "body", None);
    let node_drawing = xml_find_onechild_by_name(node_body,
        "drawing", None);
    let node_page = xml_find_onechild_by_name(node_drawing,
        "page", None);

    let node_item_set = xml_find_all_children_by_name(node_page,
        "frame", Some(("layer","layout")));

    let mut frags: Vec<Fragment> = Vec::new();

    // loop over all text frame candidates
    for node in node_item_set {
        let width_str  = node.attribute("width").unwrap_or_else(|| {  eprintln!("Error: expected 'width' attribute"); exit(1); });
        let height_str = node.attribute("height").unwrap_or_else(|| { eprintln!("Error: expected 'height' attribute"); exit(1); });
        let x_str      = node.attribute("x").unwrap_or_else(|| {      eprintln!("Error: expected 'x' attribute"); exit(1); });
        let y_str      = node.attribute("y").unwrap_or_else(|| {      eprintln!("Error: expected 'y' attribute"); exit(1); });

        let width  = parse_and_convert_cm(width_str) as i32;
        let height = parse_and_convert_cm(height_str) as i32;
        let x      = parse_and_convert_cm(x_str) as i32;
        let y      = parse_and_convert_cm(y_str) as i32;

        // navigate to the "text:span" node to get the text content
        let node_text_boxes = xml_find_all_children_by_name(node, "text-box", None);

        let node_text_box = if node_text_boxes.is_empty() {
            continue;  // was not a text-span -> ignore
        } else if node_text_boxes.len() > 1 {
            eprintln!("Bug?: multiple 'text-box' elements found?"); exit(1);
        } else {
            node_text_boxes.into_iter().next().unwrap()
        };

        // find "text:p"
        let node_p = xml_find_onechild_by_name(node_text_box, "p", None);

        // find all "text:span"
        let node_spans = xml_find_all_children_by_name(node_p, "span", None);
        // if node_spans.len()>1 { println!("SPANS: {:?}", node_spans.len()); }

        for node_span in node_spans {
            let text_style_name = node_span.attribute("style-name").unwrap_or_else(|| {
                eprintln!("Error: expected 'style-name' attribute"); exit(1); });
            let text_style_id = fontmapping[text_style_name];

            let mut text = String::new();
            if node_span.children().count()>1 {  // handle inlays of <text:s/>
                // Iterate over all node_span.children and for each extract the .text()
                for child in node_span.children() {
                    let innertext = child.text().unwrap_or("");
                    if innertext.len()>0 {
                        text.push_str(innertext);
                    }
                }
            } else {
                text.push_str(node_span.text().unwrap_or(""));
            }

            // println!("  Y:{:>5} X:{:>5}  W:{:>5} H: {:>4}  {}/{:>02}  {}", y, x, width, height, text_style_name, text_style_id, text);

            let fragment = Fragment {
                page: pagenumber,
                y, x, width, height,
                style_id: text_style_id,
                text,
            };

            frags.push(fragment);
        }
    }

    // then also find all the polygons, which are needed for table reconstruction
    let polygon_fragments: Vec<Fragment> = extract_polygons(&node_page, pagenumber);
    frags.extend(polygon_fragments);

    // sort fragments on this page by coordinate from top-bottom, left-right
    frags.sort_by(|a, b| {
        a.y.cmp(&b.y)
            .then_with(|| a.x.cmp(&b.x))
    });

    // for frag in &frags {
    //     println!("P{:>03} Y{:>5} X{:>5}  W{:>5} H{:>4}  F{:>02}  {}", frag.page, frag.y, frag.x, frag.width, frag.height, frag.style_id, frag.text);
    // }

    frags
}



/// Export/write fragments either as a list of fragments (one per line) or as plain text to a file
pub fn debug_dump_fragments_to_file(filename: &str, frags: &Vec<Fragment>, onlytext: bool, quiet: bool) -> () {
    if !quiet {
        if onlytext {
            println!("Dumping {} parsed fragments to {}", frags.len(), filename);
        } else {
            println!("Dumping {} parsed raw text to {}", frags.len(), filename);
        }
    }

    let mut buffer = String::new();

    let mut index = 0;
    while index < frags.len() {
        let frag = &frags[index];
        if onlytext {  // only raw text
            if frag.style_id != POLYGON_ID {
                if index>0 && frags[index-1].height == frag.height {
                    buffer.push_str(&format!(" {}", frag.text));
                } else {
                    buffer.push_str(&format!("\n{}", frag.text));
                }
            }
        } else { // full fragment info
            buffer.push_str(&format!(
            "i{:>06} P{:>03} Y{:>5} X{:>5}  W{:>5} H{:>4}  F{:>02}  {}\n", index,
            frag.page, frag.y, frag.x, frag.width, frag.height, frag.style_id, frag.text
        ));
        }
        index += 1;
    }

    buffer.push_str(&format!("\n"));

    match File::create(&filename) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(buffer.as_bytes()) {
                eprintln!("Error writing to file '{}': {}", filename, e);
                exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error creating file '{}': {}", filename, e);
            exit(1);
        }
    }
}
