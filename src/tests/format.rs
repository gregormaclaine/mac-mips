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
