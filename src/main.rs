use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut file: Option<String> = None;

    for i in 0..args.len() {
        let arg: &String = &args[i];

        if arg.starts_with('-') {
            if args.len() == i + 1 {
                eprintln!("Error: Invalid args");
                std::process::exit(1);
            }

            let arg_val: &String = &args[i + 1];

            match (arg.to_owned(), arg_val.to_owned()) {
                (unknown, _) => {
                    eprintln!("Error: Unknown arg, {}", unknown);
                    std::process::exit(1);
                }
            }
        } else {
            file = Some(arg.to_owned());
        }
    }

    if let Some(filename) = file {
        println!("{}", filename);
    } else {
        eprintln!("Error: Expected file as cmd line arg");
        std::process::exit(1);
    }
}
