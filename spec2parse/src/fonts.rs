// functions for keeping track of similar fonts over all the individual pages

use std::collections::BTreeMap;
use std::process::exit;

use crate::xml::*;

pub const FIRST_FONTSTYLE_ID: u32 = 1;   // Start counting a global font ID from this number, increasing

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Font {
    pub idstring: String, // a string color-name-size-weight, assembled from the other parts, for example: ffffff-Arial-27.20-bold
    pub color: String,
    pub name: String,
    pub size: String,
    pub weight: String,
}


/// Extract all the fonts information for text frames
///
/// # Arguments
/// * `node_root` - The root node of the page to parse for font styles
/// * `globalfonts` - A mutable reference to a global knowledge map of font IDs to Font structs, which will be updated (if necessary) with new fonts found on this page
pub fn parse_font_styles(node_root: &roxmltree::Node, global_fonts: &mut BTreeMap<u32, Font>) -> BTreeMap<String, u32> {
    // find font styles in FODG file
    let node_styles = xml_find_onechild_by_name(*node_root,
        "automatic-styles", None);
    let node_item_set = xml_find_all_children_by_name(node_styles,
        "style", Some(("family","text")));

    // storage for fonts in the current page
    let mut page_fonts: BTreeMap<String, u32> = BTreeMap::new();

    // find next free id in globalfonts, should we encounter a new font
    let mut next_global_font_id: u32 = FIRST_FONTSTYLE_ID;
    if let Some(&max_id) = global_fonts.keys().max() {
        next_global_font_id = max_id + 1;
    }

    // lopp over all font styles
    for node in node_item_set {
        // T1, T2, T3, ...
        let style_name = node.attribute("name").unwrap_or_else(|| {
            eprintln!("Error: expected 'name' attribute in font style?"); exit(1); });

        let text_properties = xml_find_onechild_by_name(node,
            "text-properties", None);
        let font_color = text_properties.attribute("color").unwrap_or_else(|| {
            eprintln!("Error: expected 'color' attribute"); exit(1); });
        let font_name = text_properties.attribute("font-name").unwrap_or_else(|| {
            eprintln!("Error: expected 'font-name' attribute"); exit(1); });
        let font_size_str = text_properties.attribute("font-size").unwrap_or_else(|| {
            eprintln!("Error: expected 'font-size' attribute"); exit(1); });
        let font_weight = text_properties.attribute("font-weight").unwrap_or_else(|| {
            eprintln!("Error: expected 'font-weight' attribute"); exit(1); });

        // font size cut to 2 decimal places
        let font_size_val = font_size_str.trim_end_matches("pt").parse::<f64>().unwrap_or(10.0);
        let font_size_formatted = format!("{:.2}", font_size_val);

        // create artificial long form 'idstring', for example: ffffff-Arial-27.20-bold
        let font_idstring = format!("{}-{}-{}-{}", &font_color[1..], font_name, font_size_formatted, font_weight);

        let font = Font {
            idstring: font_idstring.clone(),
            color: font_color.to_string(),
            name: font_name.to_string(),
            size: font_size_formatted.to_string(),
            weight: font_weight.to_string(),
        };

        // check if a font with the same string already exists in globalfonts
        if let Some((&id, _)) = global_fonts.iter().find(|(_, f)| f.idstring == font_idstring) {
             // if yes, just save mapping Tx -> global font id
            page_fonts.insert(style_name.to_string(), id);
        } else {
                // font never seen before, create a new global mapping
                let new_id = next_global_font_id;
                global_fonts.insert(new_id, font);
                // and save for this page also
                page_fonts.insert(style_name.to_string(), new_id);
                next_global_font_id += 1;
            }
    }

    page_fonts
}



#[allow(dead_code)]
/// print mappings of Tx -> global_font_id
pub fn debug_page_fonts(page_fonts: BTreeMap<String, u32>) -> String {
    let mut output = String::new();
    for (key, value) in &page_fonts {
        output.push_str(&format!("{}: {}\n", key, value));
    }
    output
}

#[allow(dead_code)]
/// print mappings of global_font id to font string
pub fn debug_global_fonts(globalfonts: &mut BTreeMap<u32, Font>) -> String {
    let mut output = String::new();
    for (key, value) in globalfonts {
        output.push_str(&format!("{}: {}\n", key, value.idstring));
    }
    output
}
