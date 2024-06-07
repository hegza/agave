use std::ops;

use itertools::Itertools;
use roxmltree::Document;

/// Replace bytes in a `remove_range` with bytes from `add_ranges`
pub struct ReplaceRange {
    pub remove_range: ops::Range<usize>,
    pub add_ranges: Vec<ops::Range<usize>>,
}

pub fn apply_replaces(doc: &Document, replaces: &[ReplaceRange]) -> Vec<u8> {
    let input: &[u8] = doc.input_text().as_bytes();
    let mut output: Vec<u8> = Vec::with_capacity(
        (input.len() as isize
            + replaces
                .iter()
                .map(|replace| {
                    replace.add_ranges.iter().map(|r| r.len()).sum::<usize>() as isize
                        - replace.remove_range.len() as isize
                })
                .sum::<isize>()) as usize,
    );
    // Byte cursor
    let mut cursor = 0;
    for replace in replaces {
        output.extend_from_slice(&input[cursor..replace.remove_range.start]);
        for add_range in replace.add_ranges.clone() {
            output.extend_from_slice(&input[add_range]);
        }
        cursor = replace.remove_range.end;
    }
    output.extend_from_slice(&input[cursor..]);
    output
}

pub fn apply_replaces_lines(doc: &Document, replaces: &[ReplaceRange]) -> Vec<u8> {
    let in_lines = doc
        .input_text()
        .lines()
        .map(ToString::to_string)
        .collect_vec();
    let mut output: Vec<String> = Vec::new();

    // Convert byte position to file row
    let row = |bpos| (doc.text_pos_at(bpos).row - 1) as usize;

    // Byte cursor
    let mut cursor = 0;
    for replace in replaces {
        let start_line = row(cursor);
        let end_line = row(replace.remove_range.start);
        output.extend_from_slice(&in_lines[start_line..end_line]);

        for add_range in replace.add_ranges.clone() {
            let start_line = row(add_range.start);
            let end_line = row(add_range.end) + 1;
            output.extend_from_slice(&in_lines[start_line..end_line]);
        }
        cursor = replace.remove_range.end;
    }
    let line = (doc.text_pos_at(cursor).row - 1) as usize;
    output.extend_from_slice(&in_lines[line..]);
    (output.join("\n") + "\n").as_bytes().to_vec()
}
