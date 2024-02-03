use crate::formatter;

#[test]
fn empty_file() {
    assert_eq!(formatter::format(String::new()), Ok(String::new()));
}

#[test]
fn simple_file() {
    let input = ".data\noutput: .asciiz \"Hello World\"\n.text\nmain:\nli $v0, 4\nla $a0, output\nsyscall\nend:\nli $v0, 10\nsyscall";
    let expected = ".data\n\noutput: .asciiz \"Hello World\"\n\n.text\n\nmain:\n\tli $v0, 4\n\tla $a0, output\n\tsyscall\n\nend:\n\tli $v0, 10\n\tsyscall\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn preserve_strings() {
    let should_preserve = "\"   I, am a  string\"\n";
    assert_eq!(
        formatter::format(String::from(should_preserve)),
        Ok(String::from(should_preserve))
    );

    let input = ".data\no: .asciiz \"Hello      World   ,  \"\n.text\nli $v0, 10\nsyscall";
    let expected =
        ".data\n\no: .asciiz \"Hello      World   ,  \"\n\n.text\n\nli $v0, 10\nsyscall\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn preserve_comments() {
    let input1 = "# I am a comment\n";
    assert_eq!(
        formatter::format(String::from(input1)),
        Ok(String::from(input1))
    );

    let whitespace_around_input1 = "  #   I am a comment  ";
    assert_eq!(
        formatter::format(String::from(whitespace_around_input1)),
        Ok(String::from(input1))
    );

    let input2 = "# -:1234567#890&...data###";
    let expected2 = "# -:1234567#890&...data###\n";
    assert_eq!(
        formatter::format(String::from(input2)),
        Ok(String::from(expected2))
    );
}

#[test]
fn comments_every_line() {
    let input = ".text # 1\nmain: #2\nli $v0, 1#3";
    let expected = ".text  # 1\n\nmain:  # 2\n\tli $v0, 1  # 3\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn mislaid_commas() {
    let input = "li $v0 ,1\n";
    let expected = "li $v0, 1\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn data_after_text_section() {
    let input = ".text\nmain:\nli $v0, 10\nsyscall\n.data\nZ: .word 0";
    let expected = ".text\n\nmain:\n\tli $v0, 10\n\tsyscall\n\n.data\n\nZ: .word 0\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn solo_comment_blocks() {
    let input = "# Solo Comment\n\n.text\n\n# Comment about function\nmain:\nli $v0, 1\nli $a0, 69\nsyscall";
    let expected = "# Solo Comment\n\n.text\n\n# Comment about function\nmain:\n\tli $v0, 1\n\tli $a0, 69\n\tsyscall\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn no_text_directive() {
    let input = "main:\nli $a0 , 1";
    let expected = "main:\n\tli $a0, 1\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn multiple_text_directives() {
    let input = ".text\nm:\nli $v0, 1\nli $a0, 69\n.data \n.text\nn:\nsyscall";
    let expected = ".text\n\nm:\n\tli $v0, 1\n\tli $a0, 69\n\n.data\n\n.text\n\nn:\n\tsyscall\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn linked_comment_blocks() {
    let input = "# Comment about function\nmain:\n# Middle comment\nli $v0, 1\nli $a0, 69\n\n# Linked comment\nsyscall\n";
    let expected = "# Comment about function\nmain:\n\t# Middle comment\n\tli $v0, 1\n\tli $a0, 69\n\n\t# Linked comment\n\tsyscall\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn comments_over_functions() {
    let input = "main:\nli $v0, 1\n# Comment 2\nother:\nli $v0, 1";
    let expected = "main:\n\tli $v0, 1\n\n# Comment 2\nother:\n\tli $v0, 1\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn comment_blocks_after_functions() {
    let input = "main:\nli $v0, 1\n\n# 1\n\nli $v0, 10\nsyscall\n\n# 2";
    let expected = "main:\n\tli $v0, 1\n\n\t# 1\n\n\tli $v0, 10\n\tsyscall\n\n# 2\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn files_with_globl() {
    let input = ".text\n.globl main\nmain:\nli $t2, 25";
    let expected = ".text\n\n.globl main\n\nmain:\n\tli $t2, 25\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn array_indexing() {
    let input = "lb $a0, 0 ( $sp )";
    let expected = "lb $a0, 0($sp)\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn misspaced_data() {
    let input1 = ".data\ntxt:.asciiz\"hello\"";
    let input2 = ".data\ntxt   :   .asciiz   \"hello\"";
    let expected = ".data\n\ntxt: .asciiz \"hello\"\n";
    assert_eq!(
        formatter::format(String::from(input1)),
        Ok(String::from(expected))
    );
    assert_eq!(
        formatter::format(String::from(input2)),
        Ok(String::from(expected))
    );
}

#[test]
fn long_lines_next_to_commented_lines() {
    let input =
        ".data\nlong_line: .asciiz \"A very super long string that takes up a lot of horizontal space\"\nshort_line: .space 22000  # 22KB\n";
    let expected =
        ".data\n\nlong_line: .asciiz \"A very super long string that takes up a lot of horizontal space\"\nshort_line: .space 22000  # 22KB\n";
    assert_eq!(
        formatter::format(String::from(input)),
        Ok(String::from(expected))
    );
}

#[test]
fn procedures_on_same_line_as_instruction() {
    let input1 = "main:    li $v0, 1\nother:   li $a0, 69\nsyscall";
    let input2 = "main:li $v0, 1\nother:li $a0, 69\nsyscall";
    let expected = "main:\n\tli $v0, 1\n\nother:\n\tli $a0, 69\n\tsyscall\n";
    assert_eq!(
        formatter::format(String::from(input1)),
        Ok(String::from(expected))
    );
    assert_eq!(
        formatter::format(String::from(input2)),
        Ok(String::from(expected))
    );
}

#[test]
fn directive_modifiers() {
    let input1a = ".data\n.align 2\no: .space 10";
    let input1b = ".data\n.align 2\n\n\no: .space 10";
    let expected1 = ".data\n\n.align 2\no: .space 10\n";
    assert_eq!(
        formatter::format(String::from(input1a)),
        Ok(String::from(expected1))
    );
    assert_eq!(
        formatter::format(String::from(input1b)),
        Ok(String::from(expected1))
    );

    let input2a = ".data\no: .space 10\n.align 2";
    let input2b = ".data\no: .space 10\n\n\n.align 2";
    let expected2 = ".data\n\no: .space 10\n\n.align 2\n";
    assert_eq!(
        formatter::format(String::from(input2a)),
        Ok(String::from(expected2))
    );
    assert_eq!(
        formatter::format(String::from(input2b)),
        Ok(String::from(expected2))
    );
}
