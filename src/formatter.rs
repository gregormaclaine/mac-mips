use std::fmt::Error;

use self::line::CodeLine;

static MAX_COMMENT_DISPARITY: usize = 10;

mod line {
    #[derive(Debug)]
    enum CodeToken {
        Space,
        Item(String),
        Comma,
        Colon,
        ParenOpen,
        ParenClose,
        Literal(String),
    }

    impl CodeToken {
        pub fn to_string(&self) -> String {
            return match self {
                CodeToken::Space => String::new(),
                CodeToken::Comma => String::from(","),
                CodeToken::Colon => String::from(":"),
                CodeToken::ParenOpen => String::from("("),
                CodeToken::ParenClose => String::from(")"),
                CodeToken::Item(item) => String::from(item),
                CodeToken::Literal(string) => format!("\"{}\"", string),
            };
        }

        pub fn from(c: char) -> Self {
            match c {
                ',' => CodeToken::Comma,
                ':' => CodeToken::Colon,
                '(' => CodeToken::ParenOpen,
                ')' => CodeToken::ParenClose,
                _ => panic!(),
            }
        }
    }

    fn tokenise_line(code: &str) -> Vec<CodeToken> {
        let mut tokens = vec![CodeToken::Space];

        for c in code.chars() {
            let cur_token = tokens.last_mut().unwrap();
            match (cur_token, c) {
                (CodeToken::Literal(cur), '"') if !cur.ends_with('\\') => {
                    tokens.push(CodeToken::Space)
                }
                (CodeToken::Literal(cur), c) => *cur += &c.to_string(),

                (CodeToken::Space, c) if c.is_whitespace() => {}
                (_, c) if c.is_whitespace() => tokens.push(CodeToken::Space),

                (_, ',' | ':' | '(' | ')') => tokens.push(CodeToken::from(c)),
                (_, '"') => tokens.push(CodeToken::Literal(String::new())),

                (CodeToken::Item(cur), c) => *cur += &c.to_string(),
                (_, c) => tokens.push(CodeToken::Item(c.into())),
            }
        }

        return tokens
            .into_iter()
            .filter(|t| match t {
                CodeToken::Space => false,
                _ => true,
            })
            .collect();
    }

    #[derive(Debug, Clone)]
    pub struct CodeLine {
        pub code: Option<String>,
        pub comment: Option<String>,
        pub com_gap: Option<usize>,
        indent: usize,
    }

    impl Default for CodeLine {
        fn default() -> Self {
            CodeLine::new(None, None)
        }
    }

    impl CodeLine {
        fn new(code: Option<String>, comment: Option<String>) -> Self {
            CodeLine {
                code,
                comment,
                com_gap: None,
                indent: 0,
            }
        }

        pub fn parse(line: &str) -> Self {
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

        pub fn format(&mut self) {
            if let Some(code) = &mut self.code {
                let tokens = tokenise_line(&code);
                *code = tokens[0].to_string();

                for pair in tokens.windows(2) {
                    if should_be_spaced(&pair[0], &pair[1]) {
                        *code += " ";
                    }
                    *code += &pair[1].to_string();
                }
            }
        }

        pub fn is_comment_only(&self) -> bool {
            match (&self.code, &self.comment) {
                (None, Some(_)) => true,
                _ => false,
            }
        }

        pub fn code_w(&self) -> usize {
            match &self.code {
                Some(code) => code.len(),
                None => 0,
            }
        }

        pub fn is_empty(&self) -> bool {
            match (&self.code, &self.comment) {
                (None, None) => true,
                (_, _) => false,
            }
        }

        pub fn starts_with(&self, pat: &str) -> bool {
            match &self.code {
                Some(code) => code.starts_with(pat),
                None => false,
            }
        }

        pub fn ends_with(&self, pat: &str) -> bool {
            match &self.code {
                Some(code) => code.ends_with(pat),
                None => false,
            }
        }

        pub fn indent(&mut self) {
            self.indent += 1;
        }

        pub fn set_hash_index(&mut self, h_index: usize) {
            self.com_gap = if h_index >= self.code_w() {
                Some(h_index - self.code_w())
            } else {
                None
            };
        }

        fn to_string_without_indent(&self) -> String {
            match (&self.code, &self.comment) {
                (None, None) => String::new(),
                (Some(code), None) => code.into(),
                (None, Some(comment)) => format!("# {}", comment),
                (Some(code), Some(comment)) => {
                    let comment_gap = (0..self.com_gap.unwrap_or(2))
                        .map(|_| " ")
                        .collect::<String>();
                    format!("{}{}# {}", code, comment_gap, comment)
                }
            }
        }

        pub fn to_string(&self) -> String {
            let indents: String = (0..self.indent).map(|_| "\t").collect();
            return indents + &self.to_string_without_indent();
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

    #[derive(Debug)]
    pub enum SplitLine<'a> {
        One(&'a str),
        Two((&'a str, &'a str)),
    }

    impl<'a> SplitLine<'a> {
        pub fn parse(line: &'a str) -> SplitLine<'a> {
            if let Some(colon_i) = line.find(':') {
                if let Some(hash_i) = line.find('#') {
                    if colon_i < hash_i {
                        if !&line[(colon_i + 1)..hash_i].trim().is_empty() {
                            return SplitLine::Two((&line[..=colon_i], &line[(colon_i + 1)..]));
                        }
                    }
                } else {
                    return SplitLine::Two((&line[..=colon_i], &line[(colon_i + 1)..]));
                }
            }
            return SplitLine::One(line);
        }
    }
}

#[derive(Debug)]
enum Directive {
    Text,
    Data,
}

#[derive(Debug)]
enum LineBlock<T> {
    Space,
    Code(Vec<T>),
    Data(Vec<T>),
    DataModifier(T),
    Comment(Vec<T>),
    SectionDenoter(T),
    ProcedureDenoter(T),
}

fn split_into_line_blocks(lines: &Vec<&str>) -> Vec<LineBlock<CodeLine>> {
    let mut line_blocks: Vec<LineBlock<CodeLine>> = vec![LineBlock::Space];
    let mut current_dir = Directive::Text;

    for line in lines {
        match current_dir {
            Directive::Data => consume_line(&mut line_blocks, &mut current_dir, line),
            Directive::Text => match line::SplitLine::parse(line) {
                line::SplitLine::One(line) => {
                    consume_line(&mut line_blocks, &mut current_dir, line)
                }
                line::SplitLine::Two((part1, part2)) => {
                    consume_line(&mut line_blocks, &mut current_dir, part1);
                    consume_line(&mut line_blocks, &mut current_dir, part2);
                }
            },
        }
    }

    return line_blocks;
}

fn consume_line<'a>(
    line_blocks: &mut Vec<LineBlock<CodeLine>>,
    current_dir: &mut Directive,
    line: &'a str,
) {
    let cur_block = line_blocks.last_mut().unwrap();
    let code_line = CodeLine::parse(line);

    match (cur_block, &current_dir, code_line) {
        (LineBlock::Space, _, line) if line.is_empty() => {}
        (_, _, line) if line.is_empty() => line_blocks.push(LineBlock::Space),

        (_, Directive::Data, line) if line.starts_with(".align") => {
            line_blocks.push(LineBlock::DataModifier(line));
        }

        (_, _, line) if line.starts_with(".") => {
            if line.starts_with(".text") {
                *current_dir = Directive::Text;
            } else if line.starts_with(".data") {
                *current_dir = Directive::Data;
            }
            line_blocks.push(LineBlock::SectionDenoter(line))
        }

        (_, Directive::Text, line) if line.ends_with(":") => {
            line_blocks.push(LineBlock::ProcedureDenoter(line))
        }

        (LineBlock::Comment(cur), _, line) if line.is_comment_only() => {
            cur.push(line);
        }
        (_, _, line) if line.is_comment_only() => line_blocks.push(LineBlock::Comment(vec![line])),

        (LineBlock::Code(cur) | LineBlock::Data(cur), _, line) => {
            cur.push(line);
        }
        (_, Directive::Text, line) => line_blocks.push(LineBlock::Code(vec![line])),
        (_, Directive::Data, line) => line_blocks.push(LineBlock::Data(vec![line])),
    }
}

fn comment_start_index(lines: &Vec<CodeLine>) -> usize {
    let max_length_all = lines.iter().map(|l| l.code_w()).max().unwrap_or(0);
    let max_length_comments = lines
        .iter()
        .filter_map(|l| match l.comment {
            Some(_) => Some(l.code_w()),
            None => None,
        })
        .max()
        .unwrap_or(0);

    if max_length_all - max_length_comments >= MAX_COMMENT_DISPARITY {
        max_length_comments + 2
    } else {
        max_length_all + 2
    }
}

fn format_code_block(lines: &mut Vec<CodeLine>) {
    lines.into_iter().for_each(|l| l.format());
    let comment_index = comment_start_index(&lines);
    lines
        .into_iter()
        .for_each(|l| l.set_hash_index(comment_index));
}

fn format_line_block(block: &mut LineBlock<CodeLine>) {
    match block {
        LineBlock::ProcedureDenoter(line)
        | LineBlock::SectionDenoter(line)
        | LineBlock::DataModifier(line) => line.format(),

        LineBlock::Code(lines) | LineBlock::Data(lines) => format_code_block(lines),
        LineBlock::Comment(lines) => lines.into_iter().for_each(|l| l.format()),
        LineBlock::Space => {}
    }
}

#[derive(Debug)]
enum BlockCollapseState {
    Preparing,
    AfterComment,
    AfterProcedure,
    AfterDataModifer,
}

fn consume_line_blocks<T: Default>(line_blocks: Vec<LineBlock<T>>) -> Vec<T> {
    let mut out_lines: Vec<T> = Vec::new();
    let mut state = BlockCollapseState::Preparing;

    for block in line_blocks {
        state = match (state, block) {
            (
                BlockCollapseState::AfterProcedure | BlockCollapseState::AfterDataModifer,
                LineBlock::SectionDenoter(line),
            ) => {
                out_lines.extend([T::default(), line, T::default()]);
                BlockCollapseState::Preparing
            }
            (_, LineBlock::SectionDenoter(line)) => {
                out_lines.extend([line, T::default()]);
                BlockCollapseState::Preparing
            }

            (_, LineBlock::ProcedureDenoter(line)) => {
                out_lines.push(line);
                BlockCollapseState::AfterProcedure
            }
            (_, LineBlock::Code(lines) | LineBlock::Data(lines)) => {
                out_lines.extend(lines);
                out_lines.push(T::default());
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
                out_lines.push(T::default());
                BlockCollapseState::Preparing
            }
            (state, LineBlock::Space) => state,
        };
    }

    match state {
        BlockCollapseState::Preparing => {}
        _ => out_lines.push(T::default()),
    }

    return out_lines;
}

fn indent_blocks(blocks: &mut Vec<LineBlock<CodeLine>>) {
    let mut text_chunks: Vec<Vec<&mut LineBlock<CodeLine>>> = vec![Vec::new()];
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
                match (should_indent, block) {
                    (_, LineBlock::ProcedureDenoter(_)) => should_indent = false,
                    (_, LineBlock::Code(lines)) => {
                        should_indent = true;
                        lines.into_iter().for_each(|l| l.indent());
                    }

                    (true, LineBlock::Comment(lines)) => lines.into_iter().for_each(|l| l.indent()),
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
    let mut blocks = split_into_line_blocks(&raw_lines);

    blocks.iter_mut().for_each(format_line_block);
    indent_blocks(&mut blocks);

    let lines = consume_line_blocks(blocks);

    Ok(lines
        .into_iter()
        .map(|l| l.to_string())
        .collect::<Vec<String>>()
        .join("\n"))
}
