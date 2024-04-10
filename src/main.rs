use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::{error, warn};
use owo_colors::colored::*;

use std::{
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    process,
};

fn main() {
    // handle Ctrl+C
    ctrlc::set_handler(move || {
        println!("{}", "Received Ctrl-C!".italic(),);
        process::exit(0)
    })
    .expect("Error setting Ctrl-C handler");

    // get config dir
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    // initialize the logger
    let _logger = Logger::try_with_str("info") // log warn and error
        .unwrap()
        .format_for_files(detailed_format) // use timestamp for every log
        .log_to_file(
            FileSpec::default()
                .directory(&config_dir)
                .suppress_timestamp(),
        ) // change directory for logs, no timestamps in the filename
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();

    // handle arguments
    let matches = peakfile().get_matches();
    let last_flag = matches.get_flag("last");

    if let Some(_) = matches.subcommand_matches("log") {
        if let Ok(logs) = show_log_file(&config_dir) {
            println!("{}", "Available logs:".bold().yellow());
            println!("{}", logs);
        } else {
            error!("Unable to read logs");
            process::exit(1);
        }
    } else {
        // number of lines to show defaults to 10
        let mut num_flag: u32 = 10;
        if let Some(n) = matches.get_one::<String>("num") {
            match n.parse::<u32>() {
                Ok(num) => num_flag = num,
                Err(err) => {
                    warn!("Expected an integer for the number of lines to show: {err}");
                    process::exit(0);
                }
            }
        }

        let mut content = String::new();

        let mut file = PathBuf::new();
        if let Some(arg) = matches.get_one::<String>("arg") {
            // get filepath
            file.push(&arg);
        } else {
            // read input from pipe
            let pipe_input = read_pipe();
            file.push(pipe_input);
        }

        let path = file.as_path();

        if !path.exists() {
            warn!("Path '{}' doesn`t exist", path.display());
            process::exit(0);
        }

        if !path.is_file() {
            warn!("Path '{}' is not a file", path.display());
            process::exit(0);
        }

        // read content from file
        let file_content = fs::read_to_string(path).unwrap_or_else(|err| {
            match err.kind() {
                io::ErrorKind::InvalidData => {
                    warn!("Path \'{}\' contains invalid data: {}", path.display(), err)
                }
                io::ErrorKind::NotFound => {
                    warn!("Path \'{}\' not found: {}", path.display(), err);
                }
                io::ErrorKind::PermissionDenied => {
                    warn!(
                        "Missing permission to read path \'{}\': {}",
                        path.display(),
                        err
                    )
                }
                _ => {
                    error!(
                        "Failed to access path: \'{}\'\nUnexpected error occurred: {}",
                        path.display(),
                        err
                    )
                }
            }
            process::exit(0);
        });

        content.push_str(&file_content);

        if last_flag {
            show_last_n_lines(&content, num_flag);
        } else {
            show_first_n_lines(&content, num_flag);
        }
    }
}

fn read_pipe() -> String {
    let mut input = io::stdin()
        .lock()
        .lines()
        .fold("".to_string(), |acc, line| acc + &line.unwrap() + "\n");

    let _ = input.pop();

    input.trim().to_string()
}

fn show_first_n_lines(content: &String, num_flag: u32) {
    let mut counter = 0;
    for line in content.lines() {
        if counter == num_flag {
            break;
        };

        println!("{line}");
        counter += 1;
    }
}

fn show_last_n_lines(content: &String, num_flag: u32) {
    let mut last_lines: Vec<&str> = Vec::new();
    let mut counter = 0;
    for line in content.lines().rev() {
        if counter == num_flag {
            break;
        };

        last_lines.push(line);
        counter += 1;
    }

    last_lines.iter().rev().for_each(|line| println!("{line}"));
}

// build cli
fn peakfile() -> Command {
    Command::new("pf")
        .bin_name("pf")
        .before_help(format!(
            "{}\n{}",
            "PF".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .about("Peak File")
        .before_long_help(format!(
            "{}\n{}",
            "PF".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .long_about(format!(
            "{}\n\n{}\n- {}\n- {}\n\n{}\n",
            "Peak File",
            "Show the first/last few lines of a filepath:",
            "given as an argument, or",
            "given via pipe as the output of a previous command",
            "Only works with one filepath at a time."
        ))
        // TODO update version
        .version("1.2.1")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg(
            Arg::new("arg")
                .help("The filepath to work with")
                .action(ArgAction::Set)
                .num_args(1)
                .value_name("PATH"),
        )
        .arg(
            Arg::new("last")
                .short('l')
                .long("last")
                .help("Show last n lines")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("num")
                .short('n')
                .long("num")
                .help("Number of lines to show [default: 10]")
                .action(ArgAction::Set)
                .num_args(1)
                .value_name("NUMBER"),
        )
        .subcommand(
            Command::new("log")
                .short_flag('L')
                .long_flag("log")
                .about("Show content of the log file"),
        )
}

fn check_create_config_dir() -> io::Result<PathBuf> {
    let mut new_dir = PathBuf::new();
    match dirs::config_dir() {
        Some(config_dir) => {
            new_dir.push(config_dir);
            new_dir.push("pf");
            if !new_dir.as_path().exists() {
                fs::create_dir(&new_dir)?;
            }
        }
        None => {
            error!("Unable to find config directory");
        }
    }

    Ok(new_dir)
}

fn show_log_file(config_dir: &PathBuf) -> io::Result<String> {
    let log_path = Path::new(&config_dir).join("pf.log");
    return match log_path.try_exists()? {
        true => Ok(format!(
            "{} {}\n{}",
            "Log location:".italic().dimmed(),
            &log_path.display(),
            fs::read_to_string(&log_path)?
        )),
        false => Ok(format!(
            "{} {}",
            "No log file found:"
                .truecolor(250, 0, 104)
                .bold()
                .to_string(),
            log_path.display()
        )),
    };
}
