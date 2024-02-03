use std::fmt::Error;

static MAX_COMMENT_DISPARITY: usize = 10;

mod line {
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
        Colon,
        ParenOpen,
        ParenClose,
        Literal(String),
    }

    impl CodeToken {
        fn to_string(&self) -> String {
            return match self {
                CodeToken::Comma => String::from(","),
                CodeToken::Colon => String::from(":"),
                CodeToken::ParenOpen => String::from("("),
                CodeToken::ParenClose => String::from(")"),
                CodeToken::Item(item) => String::from(item),
                CodeToken::Literal(string) => format!("\"{}\"", string),
            };
        }
    }

    fn code_token_from_char(c: char) -> CodeToken {
        match c {
            ',' => CodeToken::Comma,
            ':' => CodeToken::Colon,
            '(' => CodeToken::ParenOpen,
            ')' => CodeToken::ParenClose,
            _ => panic!(),
        }
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
            (TokenisationState::Waiting, ',' | ':' | '(' | ')') => {
                tokens.push(code_token_from_char(c));
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
            (TokenisationState::Token(cur), ',' | ':' | '(' | ')') => {
                tokens.push(CodeToken::Item(cur));
                tokens.push(code_token_from_char(c));
                return TokenisationState::Waiting;
            }
            (TokenisationState::Token(cur), '"') => {
                tokens.push(CodeToken::Item(cur));
                return TokenisationState::StringLiteral(String::new());
            }
            (TokenisationState::Token(cur), c) => TokenisationState::Token(cur + &c.to_string()),
        }
    }

    #[derive(Debug)]
    pub struct CodeLine {
        code: Option<String>,
        comment: Option<String>,
    }

    impl CodeLine {
        fn new(code: Option<String>, comment: Option<String>) -> Self {
            CodeLine { code, comment }
        }

        fn parse(line: &str) -> Self {
            if line.is_empty() {
                return CodeLine::new(None, None);
            }

            if let Some(comment_index) = line.find('#') {
                let code = line[..comment_index].trim().to_string();

                if code.is_empty() {
                    return CodeLine::new(None, Some(line[(comment_index + 1)..].trim().into()));
                }

                return CodeLine::new(Some(code), Some(line[(comment_index + 1)..].trim().into()));
            } else {
                return CodeLine::new(Some(line.trim().into()), None);
            }
        }
    }

    fn should_be_spaced(left: &CodeToken, right: &CodeToken) -> bool {
        match (left, right) {
            (
                CodeToken::Item(_) | CodeToken::Literal(_) | CodeToken::Comma | CodeToken::Colon,
                CodeToken::Item(_) | CodeToken::Literal(_),
            ) => true,
            (CodeToken::Comma, CodeToken::ParenOpen) => true,
            (_, _) => false,
        }
    }

    pub fn format_code(code: &str) -> String {
        let tokens = tokenise_line(code);

        if tokens.is_empty() {
            return String::new();
        }

        let mut formatted_code = tokens[0].to_string();

        for pair in tokens.windows(2) {
            if should_be_spaced(&pair[0], &pair[1]) {
                formatted_code += " ";
            }
            formatted_code += &pair[1].to_string();
        }

        return formatted_code;
    }

    pub fn format_comment(line: &str) -> String {
        return format!("# {}", line[1..].trim());
    }

    pub fn format(line: &str) -> String {
        let code_line = CodeLine::parse(line);
        match (code_line.code, code_line.comment) {
            (Some(code), Some(comment)) => {
                let formatted_code = format_code(&code);
                return formatted_code + "  # " + &comment;
            }
            (Some(code), None) => format_code(&code),
            (None, Some(comment)) => format_comment(&comment),
            (None, None) => String::new(),
        }
    }

    #[derive(Debug)]
    pub enum SplitLine<'a> {
        One(&'a str),
        Two((&'a str, &'a str)),
    }

    pub fn possibly_split_line<'a>(line: &'a str) -> SplitLine<'a> {
        if let Some(colon_i) = line.find(':') {
            if let Some(comment_i) = line.find('#') {
                if colon_i < comment_i {
                    return SplitLine::Two((&line[..=colon_i], &line[(colon_i + 1)..]));
                }
            } else {
                return SplitLine::Two((&line[..=colon_i], &line[(colon_i + 1)..]));
            }
        }
        return SplitLine::One(line);
    }
}

#[derive(Debug)]
enum Directive {
    Text,
    Data,
}

#[derive(Debug)]
enum Chunk {
    Space,
    Modifier(String),
    Code(Vec<String>),
    Comment(Vec<String>),
    GlobDec,
}

#[derive(Debug)]
struct Section<'a> {
    dir: Directive,
    dir_line: Option<String>,
    lines: Vec<&'a str>,
    chunks: Option<Vec<Chunk>>,
}

impl<'a> Section<'a> {
    fn new(line: &'a str, dir: Directive) -> Self {
        let dir_line = if line.is_empty() {
            None
        } else {
            Some(line::format(line))
        };

        Section {
            dir,
            dir_line,
            lines: Vec::new(),
            chunks: None,
        }
    }
}

fn parse_sections<'a>(lines: &Vec<&'a str>) -> Vec<Section<'a>> {
    let mut sections: Vec<Section> = vec![Section::new("", Directive::Text)];

    for line in lines {
        let cur_section = sections.last_mut().unwrap();
        match (&cur_section.dir, line) {
            (_, line) if line.starts_with(".text") => {
                sections.push(Section::new(line, Directive::Text));
            }
            (_, line) if line.starts_with(".data") => {
                sections.push(Section::new(line, Directive::Data));
            }
            (Directive::Data, line) => cur_section.lines.push(line),
            (Directive::Text, line) => match line::possibly_split_line(line) {
                line::SplitLine::One(line) => cur_section.lines.push(line),
                line::SplitLine::Two((part1, part2)) => cur_section.lines.extend([part1, part2]),
            },
        }
    }

    return sections;
}

fn parse_chunks<'a>(section: &mut Section<'a>) {
    let mut chunks = vec![Chunk::Space];

    for line in section.lines.iter() {
        let cur_chunk = chunks.last_mut().unwrap();
        match (cur_chunk, &section.dir, line) {
            (Chunk::Space, _, line) if line.is_empty() => {}
            (_, _, line) if line.is_empty() => chunks.push(Chunk::Space),
            (_, _, line) if line.starts_with(".globl") => chunks.push(Chunk::GlobDec),

            // === COMMENT PARSING ===
            (Chunk::Comment(cur), _, line) if line.starts_with('#') => {
                cur.push((*line).into());
            }
            (_, _, line) if line.starts_with('#') => {
                chunks.push(Chunk::Comment(vec![(*line).into()]))
            }

            // === Modifiers ===
            (_, Directive::Data, line) if line.starts_with(".align") => {
                chunks.push(Chunk::Modifier((*line).into()));
            }
            (_, Directive::Text, line) if line.ends_with(':') => {
                chunks.push(Chunk::Modifier((*line).into()));
            }

            // === STANDARD LINES ===
            (Chunk::Code(cur), _, line) => {
                cur.push((*line).into());
            }
            (_, _, line) => chunks.push(Chunk::Code(vec![(*line).into()])),
        }
    }

    section.chunks = Some(chunks);
}

// #[derive(Debug)]
// enum LineBlock<T> {
//     Space,
//     Code(Vec<T>),
//     Data(Vec<T>),
//     DataModifier(T),
//     Comment(Vec<T>),
//     SectionDenoter(T),
//     ProcedureDenoter(T),
// }

// fn split_into_line_blocks<'a>(lines: &Vec<&'a str>) -> Vec<LineBlock<&'a str>> {
//     let mut line_blocks: Vec<LineBlock<&str>> = vec![LineBlock::Space];
//     let mut current_dir = Directive::Text;

//     for line in lines {
//         match current_dir {
//             Directive::Data => consume_line(&mut line_blocks, &mut current_dir, line),
//             Directive::Text => match line::possibly_split_line(line) {
//                 line::SplitLine::One(line) => {
//                     consume_line(&mut line_blocks, &mut current_dir, line)
//                 }
//                 line::SplitLine::Two((part1, part2)) => {
//                     consume_line(&mut line_blocks, &mut current_dir, part1);
//                     consume_line(&mut line_blocks, &mut current_dir, part2);
//                 }
//             },
//         }
//     }

//     return line_blocks;
// }

// fn consume_line<'a>(
//     line_blocks: &mut Vec<LineBlock<&'a str>>,
//     current_dir: &mut Directive,
//     line: &'a str,
// ) {
//     let cur_block = line_blocks.last_mut().unwrap();
//     match (cur_block, &current_dir, line) {
//         (LineBlock::Space, _, line) if line.is_empty() => {}
//         (_, _, line) if line.is_empty() => line_blocks.push(LineBlock::Space),

//         (_, Directive::Data, line) if line.starts_with(".align") => {
//             line_blocks.push(LineBlock::DataModifier(line));
//         }

//         (_, _, line) if line.starts_with('.') => {
//             if line.starts_with(".text") {
//                 *current_dir = Directive::Text;
//             } else if line.starts_with(".data") {
//                 *current_dir = Directive::Data;
//             }
//             line_blocks.push(LineBlock::SectionDenoter(line))
//         }

//         (_, Directive::Text, line) if line.ends_with(':') => {
//             line_blocks.push(LineBlock::ProcedureDenoter(line))
//         }

//         (LineBlock::Comment(cur), _, line) if line.starts_with('#') => {
//             cur.push(line);
//         }
//         (_, _, line) if line.starts_with('#') => line_blocks.push(LineBlock::Comment(vec![line])),

//         (LineBlock::Code(cur) | LineBlock::Data(cur), _, line) => {
//             cur.push(line);
//         }
//         (_, Directive::Text, line) => line_blocks.push(LineBlock::Code(vec![line])),
//         (_, Directive::Data, line) => line_blocks.push(LineBlock::Data(vec![line])),
//     }
// }

fn comment_start_index(line_pairs: &Vec<(String, String)>) -> usize {
    let max_length_all = line_pairs.iter().map(|l| l.0.len()).max().unwrap_or(0);
    let max_length_comments = line_pairs
        .iter()
        .filter_map(|l| {
            if l.1.is_empty() {
                None
            } else {
                Some(l.0.len())
            }
        })
        .max()
        .unwrap_or(0);

    if max_length_all - max_length_comments >= MAX_COMMENT_DISPARITY {
        max_length_comments + 2
    } else {
        max_length_all + 2
    }
}

fn format_code_block(lines: &Vec<&str>) -> Vec<String> {
    let block: Vec<(String, String)> = lines
        .iter()
        .map(|l| {
            let (code, comment) = line::split_code_from_comment(l);
            (line::format_code(code), String::from(comment))
        })
        .collect();

    let comment_index = comment_start_index(&block);

    return block
        .iter()
        .map(|(code, comment)| {
            if comment.is_empty() {
                return code.clone();
            }

            let comment_indent = comment_index - code.len();
            let comment_gap = (0..comment_indent).map(|_| " ").collect::<String>();
            return format!("{}{}# {}", code, comment_gap, comment);
        })
        .collect();
}

fn format_line_block(block: LineBlock<&str>) -> LineBlock<String> {
    match block {
        LineBlock::Space => LineBlock::Space,
        LineBlock::ProcedureDenoter(line) => LineBlock::ProcedureDenoter(line::format(line)),
        LineBlock::SectionDenoter(line) => LineBlock::SectionDenoter(line::format(line)),
        LineBlock::Code(lines) => LineBlock::Code(format_code_block(&lines)),
        LineBlock::Data(lines) => LineBlock::Data(format_code_block(&lines)),
        LineBlock::DataModifier(line) => LineBlock::DataModifier(line::format(line)),
        LineBlock::Comment(lines) => {
            LineBlock::Comment(lines.into_iter().map(line::format_comment).collect())
        }
    }
}

#[derive(Debug)]
enum BlockCollapseState {
    Preparing,
    AfterComment,
    AfterProcedure,
    AfterDataModifer,
}

fn consume_line_blocks<T: for<'a> From<&'a str>>(line_blocks: Vec<LineBlock<T>>) -> Vec<T> {
    let mut out_lines: Vec<T> = Vec::new();
    let mut state = BlockCollapseState::Preparing;

    for block in line_blocks {
        state = match (state, block) {
            (
                BlockCollapseState::AfterProcedure | BlockCollapseState::AfterDataModifer,
                LineBlock::SectionDenoter(line),
            ) => {
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
            (_, LineBlock::Code(lines) | LineBlock::Data(lines)) => {
                out_lines.extend(lines);
                out_lines.push("".into());
                BlockCollapseState::Preparing
            }
            (_, LineBlock::Comment(lines)) => {
                out_lines.extend(lines);
                BlockCollapseState::AfterComment
            }
            (_, LineBlock::DataModifier(line)) => {
                out_lines.push(line);
                BlockCollapseState::AfterDataModifer
            }

            (BlockCollapseState::AfterComment, LineBlock::Space) => {
                out_lines.push("".into());
                BlockCollapseState::Preparing
            }
            (state, LineBlock::Space) => state,
        };
    }

    match state {
        BlockCollapseState::Preparing => {}
        _ => out_lines.push("".into()),
    }

    return out_lines;
}

fn indent_block(block: &mut LineBlock<String>) {
    match block {
        LineBlock::Code(lines) | LineBlock::Comment(lines) => {
            for line in lines {
                *line = String::from("\t") + line;
            }
        }

        // Directives & Procedures should never be indented
        LineBlock::Space
        | LineBlock::ProcedureDenoter(_)
        | LineBlock::SectionDenoter(_)
        | LineBlock::Data(_)
        | LineBlock::DataModifier(_) => {}
    }
}

fn indent_blocks(blocks: &mut Vec<LineBlock<String>>) {
    let mut text_chunks: Vec<Vec<&mut LineBlock<String>>> = vec![Vec::new()];
    let mut in_data_chunk = false;

    for block in blocks {
        match (in_data_chunk, block) {
            (_, LineBlock::SectionDenoter(line)) if line.starts_with(".data") => {
                in_data_chunk = true;
            }
            (_, LineBlock::SectionDenoter(line)) if line.starts_with(".text") => {
                in_data_chunk = false;
                text_chunks.push(Vec::new());
            }
            (false, block) => text_chunks.last_mut().unwrap().push(block),
            (_, _) => {}
        }
    }

    for chunk in text_chunks {
        let first_proc_index = chunk.iter().enumerate().find_map(|(i, b)| match b {
            LineBlock::ProcedureDenoter(_) => Some(i),
            _ => None,
        });

        if let Some(index) = first_proc_index {
            let mut should_indent = false;

            for block in chunk.into_iter().skip(index + 1).rev() {
                match (should_indent, &block) {
                    (_, LineBlock::ProcedureDenoter(_)) => should_indent = false,
                    (_, LineBlock::Code(_)) => {
                        should_indent = true;
                        indent_block(block);
                    }

                    (true, LineBlock::Comment(_)) => indent_block(block),
                    (false, LineBlock::Comment(_)) => {}

                    (
                        _,
                        LineBlock::Space
                        | LineBlock::SectionDenoter(_)
                        | LineBlock::Data(_)
                        | LineBlock::DataModifier(_),
                    ) => {}
                }
            }
        }
    }
}

pub fn format(contents: String) -> Result<String, Error> {
    let raw_lines: Vec<&str> = contents.lines().map(|l| l.trim()).collect();

    let mut sections = parse_sections(&raw_lines);
    for section in sections {
        parse_chunks(&mut section);
    }

    // let blocks: Vec<LineBlock<&str>> = split_into_line_blocks(&raw_lines);

    let mut formatted_blocks: Vec<LineBlock<String>> =
        blocks.into_iter().map(format_line_block).collect();

    indent_blocks(&mut formatted_blocks);

    let lines: Vec<String> = consume_line_blocks(formatted_blocks);

    Ok(lines.join("\n"))
}
