mod formatter;

#[cfg(test)]
mod tests;

use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

fn help() {
    println!("mac-mips v0.1.0\n\nUsage:");
    println!("\tmacmips [filename] [args]\n");
    println!("Arguments:");
    println!("\t-h\t        See docs about tool");
    println!("\t-o <OUT DIR>\tOutput directory");
    std::process::exit(0);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut file: Option<String> = None;
    let mut output_dir: Option<&str> = None;

    let mut i = 1;
    while i < args.len() {
        let arg: &str = args[i].as_str();

        if arg.starts_with('-') {
            let arg_val: Option<&String> = args.get(i + 1);

            match (arg, arg_val) {
                ("-h", _) => help(),
                ("-o", Some(output)) => {
                    output_dir = Some(output.as_str());
                    i += 1;
                }
                (unknown, _) => {
                    eprintln!("Error: Invalid args, {}", unknown);
                    std::process::exit(1);
                }
            };
        } else {
            file = Some(arg.to_owned());
        }

        i += 1
    }

    if let Some(filename) = file {
        let path = Path::new(filename.as_str());
        let file = fs::read_to_string(path);

        if let Err(e) = file {
            eprintln!("Error: Couldn't read file");
            eprintln!("{}", e);
            std::process::exit(1);
        }

        let contents = file.unwrap();
        let formatted = formatter::format(contents);

        if let Err(e) = formatted {
            eprintln!("Error: Couldn't format file");
            eprintln!("{}", e);
            std::process::exit(1);
        }

        let formatted_content = formatted.unwrap();

        let out_path = match output_dir {
            Some(outdir) => Path::new(outdir).join(path.file_name().unwrap()),
            None => path.to_path_buf(),
        };

        let file = fs::File::create(out_path);

        if let Err(e) = file {
            eprintln!("Error: Couldn't edit file");
            eprintln!("{}", e);
            std::process::exit(1);
        }

        if let Err(e) = file.unwrap().write_all(formatted_content.as_bytes()) {
            eprintln!("Error: Couldn't write formatted code to file");
            eprintln!("{}", e);
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: Expected file as cmd line arg");
        eprintln!("       To see how to use this tool, use 'macmips -h'");
        std::process::exit(1);
    }
}
