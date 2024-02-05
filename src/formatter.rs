use std::fmt::Error;

use self::line::CodeLine;
use self::line::SplitLine;

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
enum Chunk {
    Space,
    GlobDec(CodeLine),
    Modifier(CodeLine),
    Code(Vec<CodeLine>),
    Comment(Vec<CodeLine>),
}

#[derive(Debug)]
struct Section {
    dir: Directive,
    dir_line: Option<CodeLine>,
    lines: Vec<CodeLine>,
}

impl Section {
    fn new(line: &str, dir: Directive) -> Self {
        let dir_line = if line.is_empty() {
            None
        } else {
            Some(CodeLine::parse(line))
        };

        Section {
            dir,
            dir_line,
            lines: Vec::new(),
        }
    }
}

fn parse_sections(lines: &Vec<&str>) -> Vec<Section> {
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
            (Directive::Data, line) => cur_section.lines.push(CodeLine::parse(line)),
            (Directive::Text, line) => match SplitLine::parse(line) {
                SplitLine::One(line) => cur_section.lines.push(CodeLine::parse(line)),
                SplitLine::Two((part1, part2)) => cur_section
                    .lines
                    .extend([CodeLine::parse(part1), CodeLine::parse(part2)]),
            },
        }
    }

    return sections;
}

fn parse_chunks(lines: Vec<CodeLine>, dir: &Directive) -> Vec<Chunk> {
    let mut chunks = vec![Chunk::Space];

    for line in lines {
        let cur_chunk = chunks.last_mut().unwrap();
        match (cur_chunk, dir, line) {
            (Chunk::Space, _, line) if line.is_empty() => {}
            (_, _, line) if line.is_empty() => chunks.push(Chunk::Space),
            (_, _, line) if line.starts_with(".globl") => chunks.push(Chunk::GlobDec(line)),

            // === COMMENT PARSING ===
            (Chunk::Comment(cur), _, line) if line.is_comment_only() => {
                cur.push(line);
            }
            (_, _, line) if line.is_comment_only() => chunks.push(Chunk::Comment(vec![line])),

            // === Modifiers ===
            (_, Directive::Data, line) if line.starts_with(".align") => {
                chunks.push(Chunk::Modifier(line));
            }
            (_, Directive::Text, line) if line.ends_with(":") => {
                chunks.push(Chunk::Modifier(line));
            }

            // === STANDARD LINES ===
            (Chunk::Code(cur), _, line) => {
                cur.push(line);
            }
            (_, _, line) => chunks.push(Chunk::Code(vec![line])),
        }
    }

    return chunks;
}

fn calc_hash_index(lines: &Vec<CodeLine>) -> usize {
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

fn align_comments(chunk: &mut Chunk) {
    if let Chunk::Code(lines) = chunk {
        let comment_index = calc_hash_index(&lines);
        lines
            .into_iter()
            .for_each(|l| l.set_hash_index(comment_index));
    }
}

fn indent_chunks(chunks: &mut Vec<Chunk>) {
    let first_proc_index = chunks.iter().enumerate().find_map(|(i, b)| match b {
        Chunk::Modifier(_) => Some(i),
        _ => None,
    });

    if let Some(index) = first_proc_index {
        let mut should_indent = false;

        for block in chunks.into_iter().skip(index + 1).rev() {
            match (should_indent, block) {
                (_, Chunk::Modifier(_)) => should_indent = false,
                (_, Chunk::Code(lines)) => {
                    should_indent = true;
                    lines.into_iter().for_each(|l| l.indent());
                }

                (true, Chunk::Comment(lines)) => lines.into_iter().for_each(|l| l.indent()),
                (false, Chunk::Comment(_)) => {}

                (_, Chunk::Space | Chunk::GlobDec(_)) => {}
            }
        }
    }
}

#[derive(Debug)]
enum CompileState {
    Free,
    AfterComment,
    AfterModifier,
}

fn compile_section(lines: &mut Vec<CodeLine>, dir_line: Option<CodeLine>, chunks: Vec<Chunk>) {
    if let Some(dir_line) = dir_line {
        lines.extend([dir_line, CodeLine::default()]);
    }

    let mut state = CompileState::Free;

    for block in chunks {
        state = match (state, block) {
            (CompileState::Free, Chunk::GlobDec(line)) => {
                lines.extend([line, CodeLine::default()]);
                CompileState::Free
            }
            (_, Chunk::GlobDec(line)) => {
                lines.extend([CodeLine::default(), line, CodeLine::default()]);
                CompileState::Free
            }

            (_, Chunk::Code(_lines)) => {
                lines.extend(_lines);
                lines.push(CodeLine::default());
                CompileState::Free
            }
            (_, Chunk::Comment(_lines)) => {
                lines.extend(_lines);
                CompileState::AfterComment
            }
            (_, Chunk::Modifier(line)) => {
                lines.push(line);
                CompileState::AfterModifier
            }

            (CompileState::AfterComment, Chunk::Space) => {
                lines.push(CodeLine::default());
                CompileState::Free
            }
            (state, Chunk::Space) => state,
        };
    }

    match state {
        CompileState::Free => {}
        _ => lines.push(CodeLine::default()),
    }
}

pub fn format(contents: String) -> Result<String, Error> {
    let raw_lines: Vec<&str> = contents.lines().map(|l| l.trim()).collect();
    let sections = parse_sections(&raw_lines);
    let mut output_lines: Vec<CodeLine> = Vec::new();

    for section in sections {
        // === Formatting ===
        let mut lines = section.lines;
        lines.iter_mut().for_each(|l| l.format());
        let mut chunks = parse_chunks(lines, &section.dir);
        chunks.iter_mut().for_each(|c| align_comments(c));

        match &section.dir {
            Directive::Text => indent_chunks(&mut chunks),
            Directive::Data => {}
        }

        // === Compilation ===
        compile_section(&mut output_lines, section.dir_line, chunks);
    }

    Ok(output_lines
        .into_iter()
        .map(|l| l.to_string())
        .collect::<Vec<String>>()
        .join("\n"))
}
