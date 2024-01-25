use std::fmt::Error;

pub fn format(contents: String) -> Result<String, Error> {
    let raw_lines: Vec<&str> = contents.lines().map(|l| l.trim()).collect();

    let mut line_blocks: Vec<Vec<&str>> = vec![Vec::new()];

    for line in raw_lines {
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

    let mut lines: Vec<&str> = Vec::new();

    for block in line_blocks {
        for line in block {
            lines.push(line);
        }
        lines.push("");
    }

    let mut formatted_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            let mut parts: Vec<String> = line.split_whitespace().map(|p| String::from(p)).collect();
            for i in (0..parts.len()).rev() {
                if parts[i] == "," {
                    parts.remove(i);
                    parts[i - 1] += ",";
                }
            }
            return parts.join(" ");
        })
        .collect();

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
