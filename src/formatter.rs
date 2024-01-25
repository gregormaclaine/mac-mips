use std::fmt::Error;

fn remove_redundant_lines<'a>(lines: &'a Vec<&'a str>) -> Vec<&'a str> {
    let mut line_blocks: Vec<Vec<&str>> = vec![Vec::new()];

    for line in lines {
        if let Some(cur_block) = line_blocks.last_mut() {
            if line.is_empty() {
                if cur_block.len() != 0 && !(cur_block[0].ends_with(":") && cur_block.len() == 1) {
                    line_blocks.push(Vec::new());
                }
                continue;
            }

            if line.starts_with(".") {
                if cur_block.len() != 0 {
                    line_blocks.push(vec![line]);
                } else {
                    cur_block.push(line);
                }
                line_blocks.push(Vec::new());
                continue;
            }

            if line.ends_with(":") {
                if cur_block.len() != 0 {
                    line_blocks.push(vec![line]);
                } else {
                    cur_block.push(line);
                }
                continue;
            }

            cur_block.push(line);
        }
    }

    let mut output: Vec<&str> = Vec::new();

    for block in line_blocks {
        if block.len() == 0 {
            continue;
        }
        for line in block {
            output.push(line);
        }
        output.push("");
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

fn format_line(line: &str) -> String {
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
        return parts.join(" ") + "  # " + comment.trim();
    }
}

pub fn format(contents: String) -> Result<String, Error> {
    let raw_lines: Vec<&str> = contents.lines().map(|l| l.trim()).collect();

    let lines = remove_redundant_lines(&raw_lines);

    let mut formatted_lines: Vec<String> = lines.iter().map(|line| format_line(line)).collect();

    let mut after_bookmarks = false;
    for line in formatted_lines.iter_mut() {
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

    return Ok(formatted_lines.join("\n"));
}
