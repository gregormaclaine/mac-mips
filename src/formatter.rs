use std::fmt::Error;

fn remove_redundant_lines(lines: &Vec<&str>) -> Vec<String> {
    let mut line_blocks: Vec<Vec<&str>> = vec![Vec::new()];

    for line in lines {
        if let Some(cur_block) = line_blocks.last_mut() {
            if line.is_empty() {
                if cur_block.len() != 0 && !(cur_block[0].ends_with(":") && cur_block.len() == 1) {
                    line_blocks.push(Vec::new());
                }
                continue;
            }

            if line.starts_with(".") || line.ends_with(":") {
                if cur_block.len() != 0 {
                    line_blocks.push(vec![line]);
                } else {
                    cur_block.push(line);
                }
                line_blocks.push(Vec::new());
                continue;
            }

            cur_block.push(line);
        }
    }

    let mut output: Vec<String> = Vec::new();

    for block in line_blocks {
        if block.len() == 0 {
            continue;
        }

        if block[0].ends_with(":") {
            output.push(String::from(block[0]));
            continue;
        }

        let formatted_block: Vec<String> = block.iter().map(|l| format_line(l, 2)).collect();

        let max_length = get_max_length(&formatted_block);

        for line in formatted_block {
            let (code, _) = split_line_from_comment(line.as_str());
            output.push(format_line(
                line.as_str(),
                max_length - code.len() as u32 + 2,
            ));
        }

        output.push(String::new());
    }

    return output;
}

fn split_line_from_comment(line: &str) -> (&str, &str) {
    if let Some(comment_index) = line.find('#') {
        return (&line[..comment_index], &line[(comment_index + 1)..]);
    } else {
        return (line, "");
    }
}

fn get_max_length(lines: &Vec<String>) -> u32 {
    return lines
        .iter()
        .map(|l| {
            let (code, _) = split_line_from_comment(l);
            return code.len();
        })
        .max()
        .unwrap() as u32;
}

fn format_line(line: &str, comment_indent: u32) -> String {
    let (code, comment) = split_line_from_comment(line);

    let mut parts: Vec<String> = code.split_whitespace().map(|p| String::from(p)).collect();
    for i in (0..parts.len()).rev() {
        if parts[i] == "," {
            parts.remove(i);
            parts[i - 1] += ",";
        }
    }

    if comment == "" {
        return parts.join(" ");
    } else {
        let comment_gap = (0..comment_indent).map(|_| " ").collect::<String>();
        return parts.join(" ") + comment_gap.as_str() + "# " + comment.trim();
    }
}

pub fn format(contents: String) -> Result<String, Error> {
    let raw_lines: Vec<&str> = contents.lines().map(|l| l.trim()).collect();

    let mut lines = remove_redundant_lines(&raw_lines);

    let mut after_bookmarks = false;
    for line in lines.iter_mut() {
        if !after_bookmarks {
            if line.ends_with(":") {
                after_bookmarks = true;
            }
            continue;
        }

        if !line.is_empty() && !line.ends_with(":") {
            *line = String::from("\t") + line;
        }
    }

    return Ok(lines.join("\n"));
}
