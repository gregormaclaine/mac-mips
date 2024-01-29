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
}

#[test]
fn collapse_comments() {
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
