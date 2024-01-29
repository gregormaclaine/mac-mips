use std::fmt::Error;

#[derive(Debug)]
enum LineBlock<T> {
    Space,
    Code(Vec<T>),
    Comment(Vec<T>),
    SectionDenoter(T),
    ProcedureDenoter(T),
}

fn split_into_line_blocks<'a>(lines: &Vec<&'a str>) -> Vec<LineBlock<&'a str>> {
    let mut line_blocks: Vec<LineBlock<&str>> = vec![LineBlock::Space];

    for line in lines {
        if let Some(cur_block) = line_blocks.last_mut() {
            match (cur_block, line) {
                (LineBlock::Space, line) if line.is_empty() => {}
                (_, line) if line.is_empty() => line_blocks.push(LineBlock::Space),

                (_, line) if line.starts_with('.') => {
                    line_blocks.push(LineBlock::SectionDenoter(line))
                }

                (_, line) if line.ends_with(':') => {
                    line_blocks.push(LineBlock::ProcedureDenoter(line))
                }

                (LineBlock::Comment(cur), line) if line.starts_with('#') => {
                    cur.push(line);
                }
                (_, line) if line.starts_with('#') => {
                    line_blocks.push(LineBlock::Comment(vec![line]))
                }

                (LineBlock::Code(cur), line) => {
                    cur.push(line);
                }
                (_, line) => line_blocks.push(LineBlock::Code(vec![line])),
            }
        }
    }

    return line_blocks;
}

fn format_line_block(block: LineBlock<&str>) -> LineBlock<String> {
    match block {
        LineBlock::Space => LineBlock::Space,
        LineBlock::ProcedureDenoter(line) => LineBlock::ProcedureDenoter(format_solo_line(line)),
        LineBlock::SectionDenoter(line) => LineBlock::SectionDenoter(format_solo_line(line)),
        LineBlock::Code(lines) => LineBlock::Code(format_code_block(&lines)),
        LineBlock::Comment(lines) => {
            LineBlock::Comment(lines.into_iter().map(format_comment).collect())
        }
    }
}

#[derive(Debug)]
enum BlockCollapseState {
    Preparing,
    AfterComment,
    AfterProcedure,
}

fn consume_line_blocks<T: for<'a> From<&'a str>>(line_blocks: Vec<LineBlock<T>>) -> Vec<T> {
    let mut out_lines: Vec<T> = Vec::new();
    let mut state = BlockCollapseState::Preparing;

    for block in line_blocks {
        state = match (state, block) {
            (BlockCollapseState::AfterProcedure, LineBlock::SectionDenoter(line)) => {
                out_lines.extend(["".into(), line, "".into()]);
                BlockCollapseState::Preparing
            }
            (_, LineBlock::SectionDenoter(line)) => {
                out_lines.extend([line, "".into()]);
                BlockCollapseState::Preparing
            }

            (_, LineBlock::ProcedureDenoter(line)) => {
                out_lines.push(line);
                BlockCollapseState::AfterProcedure
            }
            (_, LineBlock::Code(lines)) => {
                out_lines.extend(lines);
                out_lines.push("".into());
                BlockCollapseState::Preparing
            }
            (_, LineBlock::Comment(lines)) => {
                out_lines.extend(lines);
                BlockCollapseState::AfterComment
            }

            (BlockCollapseState::Preparing, LineBlock::Space) => BlockCollapseState::Preparing,
            (BlockCollapseState::AfterComment, LineBlock::Space) => {
                out_lines.push("".into());
                BlockCollapseState::Preparing
            }

            (BlockCollapseState::AfterProcedure, LineBlock::Space) => {
                BlockCollapseState::AfterProcedure
            }
        };
    }

    match state {
        BlockCollapseState::Preparing => {}
        _ => out_lines.push("".into()),
    }

    return out_lines;
}

fn format_comment(line: &str) -> String {
    return format!("# {}", line[1..].trim());
}

fn format_code_block(lines: &Vec<&str>) -> Vec<String> {
    let block: Vec<(String, &str)> = lines
        .iter()
        .map(|l| {
            let (code, comment) = split_line_from_comment(l);
            (format_line_of_code(code), comment.trim())
        })
        .collect();

    let max_length = block.iter().map(|l| l.0.len()).max().unwrap_or(0);

    return block
        .iter()
        .map(|(code, comment)| {
            if comment.is_empty() {
                return code.clone();
            }

            let comment_indent = max_length - code.len() + 2;
            let comment_gap = (0..comment_indent).map(|_| " ").collect::<String>();
            return format!("{}{}# {}", code, comment_gap, comment);
        })
        .collect();
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

#[derive(Debug)]
enum TokenisationState {
    Token(String),
    StringLiteral(String),
    Waiting,
}

#[derive(Debug)]
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

fn format_solo_line(line: &str) -> String {
    let (code, comment) = split_line_from_comment(line);
    let formatted_code = format_line_of_code(code);

    if comment.is_empty() {
        return formatted_code;
    }

    return formatted_code + " # " + comment.trim();
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
    let blocks = split_into_line_blocks(&raw_lines);
    let formatted_blocks: Vec<LineBlock<String>> =
        blocks.into_iter().map(format_line_block).collect();

    let mut lines = consume_line_blocks(formatted_blocks);

    indent_lines_after_procedures(&mut lines);
    Ok(lines.join("\n"))
}
