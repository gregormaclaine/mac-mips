use std::fmt::Error;

fn split_into_line_blocks<'a>(lines: &Vec<&'a str>) -> Vec<Vec<&'a str>> {
    let mut line_blocks: Vec<Vec<&str>> = vec![Vec::new()];

    for line in lines {
        if let Some(cur_block) = line_blocks.last_mut() {
            if line.is_empty() {
                if !cur_block.is_empty() {
                    line_blocks.push(Vec::new());
                }
                continue;
            }

            if line.starts_with('.') || line.ends_with(':') {
                if !cur_block.is_empty() {
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

    return line_blocks;
}

fn collapse_line_blocks(line_blocks: &Vec<Vec<&str>>) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();

    for block in line_blocks {
        if block.is_empty() {
            continue;
        }

        if block[0].ends_with(':') {
            output.push(String::from(block[0]));
            continue;
        }

        let formatted_block: Vec<String> = block.iter().map(|l| format_line(l, 2)).collect();

        let max_length = get_max_code_length(&formatted_block);

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
        return (
            &line[..comment_index].trim(),
            &line[(comment_index + 1)..].trim(),
        );
    } else {
        return (line.trim(), "");
    }
}

fn get_max_code_length(lines: &[String]) -> u32 {
    return lines
        .iter()
        .map(|l| split_line_from_comment(l).0.len())
        .max()
        .unwrap() as u32;
}

enum TokenisationState {
    Token(String),
    StringLiteral(String),
    Waiting,
}

enum CodeToken {
    Item(String),
    Comma,
    Literal(String),
}

fn tokenise_line(code: &str) -> Vec<CodeToken> {
    let mut tokens: Vec<CodeToken> = Vec::new();
    let mut state = TokenisationState::Waiting;

    for c in code.chars() {
        state = receive_next_char(state, c, &mut tokens);
    }

    match state {
        TokenisationState::Waiting => {}
        TokenisationState::Token(cur) => tokens.push(CodeToken::Item(cur)),
        TokenisationState::StringLiteral(cur) => tokens.push(CodeToken::Literal(cur)),
    }

    return tokens;
}

fn receive_next_char(
    state: TokenisationState,
    c: char,
    tokens: &mut Vec<CodeToken>,
) -> TokenisationState {
    match (state, c) {
        (TokenisationState::Waiting, ',') => {
            tokens.push(CodeToken::Comma);
            return TokenisationState::Waiting;
        }
        (TokenisationState::Waiting, c) if c.is_whitespace() => TokenisationState::Waiting,
        (TokenisationState::Waiting, '"') => TokenisationState::StringLiteral(String::new()),
        (TokenisationState::Waiting, c) => TokenisationState::Token(String::from(c)),

        (TokenisationState::StringLiteral(cur), '"') if !cur.ends_with('\\') => {
            tokens.push(CodeToken::Literal(cur));
            return TokenisationState::Waiting;
        }
        (TokenisationState::StringLiteral(cur), c) => {
            TokenisationState::StringLiteral(cur + &c.to_string())
        }

        (TokenisationState::Token(cur), c) if c.is_whitespace() => {
            tokens.push(CodeToken::Item(cur));
            return TokenisationState::Waiting;
        }
        (TokenisationState::Token(cur), ',') => {
            tokens.push(CodeToken::Item(cur));
            tokens.push(CodeToken::Comma);
            return TokenisationState::Waiting;
        }
        (TokenisationState::Token(cur), c) => TokenisationState::Token(cur + &c.to_string()),
    }
}

fn format_line_of_code(code: &str) -> String {
    let tokens = tokenise_line(code);

    let mut formatted_code = String::new();

    for t in tokens {
        match t {
            CodeToken::Comma => formatted_code += ",",
            CodeToken::Item(item) => formatted_code += &(" ".to_owned() + &item),
            CodeToken::Literal(string) => formatted_code += &format!("{}{}{}", " \"", string, '"'),
        }
    }

    return formatted_code.trim_start().to_string();
}

fn format_line(line: &str, comment_indent: u32) -> String {
    let (code, comment) = split_line_from_comment(line);
    let formatted_code = format_line_of_code(code);

    if comment.is_empty() {
        return formatted_code;
    }

    if code.is_empty() {
        return String::from("# ") + comment.trim();
    }

    let comment_gap = (0..comment_indent).map(|_| " ").collect::<String>();
    return formatted_code + comment_gap.as_str() + "# " + comment.trim();
}

fn indent_lines_after_procedures(lines: &mut Vec<String>) {
    for line in lines
        .iter_mut()
        .skip_while(|l| !l.starts_with(".text"))
        .skip_while(|l| !l.ends_with(':'))
    {
        if line.starts_with('.') {
            break;
        }

        if !line.is_empty() && !line.ends_with(':') {
            *line = String::from("\t") + line;
        }
    }
}

pub fn format(contents: String) -> Result<String, Error> {
    let raw_lines: Vec<&str> = contents.lines().map(|l| l.trim()).collect();
    let line_blocks = split_into_line_blocks(&raw_lines);
    let mut lines = collapse_line_blocks(&line_blocks);
    indent_lines_after_procedures(&mut lines);
    Ok(lines.join("\n"))
}
