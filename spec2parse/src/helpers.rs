/// fake index which indicates end of data/file
pub const END_OF_DATA: usize = 999999;

/// virtual ID number to denote a polygon fragment (instead of a text fragment)
pub const POLYGON_ID: u32 = 90;

// fill in texts (for human understanding) for the polygon pieces found
pub const SQUARE: &str = "■";
pub const HORI: &str = "━";
pub const VERT: &str = "┃";


/// Parses a selection of ranges
///
/// * `pages` - A string describing the selection
///   Examples:
///   - "all" - Select all
///   - "1" - Select only 1
///   - "1,3-5,7" - Select 1, 3 through 5, and 7
///   - Note: open range "9-" is not supported (because this function does not know what the maximum number is)
///
/// # Returns
/// A vector containing the numbers selected.
pub fn parse_selection(selection: &str) -> Vec<usize> {
    let mut selected_numbers = Vec::new();
    let ranges = selection.split(',');

    for range in ranges {
        let range = range.trim();
        if range.contains('-') {
            // it's a range like "1-3"
            let mut parts = range.split('-');
            let start = parts.next().unwrap().parse::<usize>().unwrap();
            let end = parts.next().unwrap().parse::<usize>().unwrap();

            for i in start..=end {
                selected_numbers.push(i);
            }
        } else {
            // it's a single number like "1"
            let page_num = range.parse::<usize>().unwrap();
            selected_numbers.push(page_num);
        }
    }
    // println!("{:?}", selected_numbers);

    selected_numbers
}


