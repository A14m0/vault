use clap::{Arg, App};
use colored::Colorize;
use std::io::prelude::*;

mod datafile;
use datafile::Datafile;


/// structure that defines a Command for the CLI interface
struct Command {
    value: &'static str,
    help: &'static str,
    command: fn(String, &mut Datafile) -> u32
}

/// Our constant list of commands
const COMMS: [Command; 9] = [
        Command{
            value: "exit",
            help: "Exits the program",
            command: exit
        },
        Command{
            value: "ls",
            help: "Lists all the files in the archive",
            command: ls
        },
        Command{
            value: "load",
            help: "Loads a new file for interaction",
            command: load
        },
        Command{
            value: "save",
            help: "Saves the current state to a file",
            command: save
        },
        Command{
            value: "add",
            help: "Adds a file to the current state",
            command: add
        },
        Command{
            value: "remove",
            help: "Removes a file from the current state",
            command: ls
        },
        Command{
            value: "fetch",
            help: "Fetches a file from the current state and saves it to the filesystem",
            command: ls
        },
        Command{
            value: "pass",
            help: "Updates the currently used passphrase for encryption",
            command: pass
        },
        Command{
            value: "help",
            help: "Prints all the help information for the commands",
            command: help
        }
    ];

////////////// COMMAND FUNCTIONS ///////////////
/// Exits the program
fn exit(_args: String, _dfile: &mut Datafile) -> u32 {
    // make sure the user really wants to exit the program
    print!("{}", "[ ] Are you sure you wish to exit? (y/n) > ");
    std::io::stdout().flush().unwrap();
    
    let mut ret = String::new();
    std::io::stdin().read_line(&mut ret).expect("Failed to read STDIN");
    let ret = ret.replace("\n", "");

    // figure out the user's response
    match ret.as_str() {
        "y" => {
            
            println!("{}", "[+] Exiting...".green());

            // exit the program
            std::process::exit(0);
        },
        &_ => {
            println!("{}", "[ ] Not exiting".green());
        }
    }
    0
}

/// lists all files in the datafile
fn ls(_args: String, dfile: &mut Datafile) -> u32 {
    println!("{} {} {}", "[+] Currently".green(), dfile.num_files(), "files".green());
    // loop over each of the files
    for file in dfile.files().iter() {
        println!("\t{}", file);
    }

    0
}

/// loads a new file
fn load(_args: String, _dfile: &mut Datafile) -> u32 {
    0
}

/// saves to a file
fn save(args: String, dfile: &mut Datafile) -> u32 {
    // see if we got a path
    if !args.is_empty() {
        let path: &str;
        // see if we can save to the first path provided by the arguments
        if args.contains(" ") {
            let space_index = args.as_bytes().iter().position(|&r| r == b' ').unwrap();
            path = &args[..space_index];
        } else {
            path = &args[..];
        }

        println!("{}: {}","[ ] Saving file to", path);
        match dfile.save(path.to_string()) {
            Ok(_) => return 0,
            Err(e) => println!("{}: {}", "[-] Failed to save file".red(), e)
        };
    } 

    // assuming something doesnt work or we dont get args, we just loop to try 
    // and save the user's file
    loop {
        print!("{}", "[ ] Enter path to save file to > ");
        std::io::stdout().flush().unwrap();
    
        let mut r = String::new();
        std::io::stdin().read_line(&mut r).expect("Failed to read STDIN");
        
        // try to save the file
        match dfile.save(r) {
            Ok(_) => return 0,
            Err(e) => println!("{}: {}", "[-] Failed to save file".red(), e)
        };
    }
}

/// adds a file
fn add(_args: String, _dfile: &mut Datafile) -> u32 {
    0
}

/// removes a file
#[allow(dead_code)]
fn remove(_args: String, _dfile: &mut Datafile) -> u32 {
    0
}

/// fetches a file and stores it wherever the user wants it to be stored
#[allow(dead_code)]
fn fetch(_args: String, _dfile: &mut Datafile) -> u32 {
    0
}

/// updates the current password
fn pass(args: String, dfile: &mut Datafile) -> u32 {
    if !args.is_empty() {
        let pass: &str;
        // see if we can save to the first path provided by the arguments
        if args.contains(" ") {
            println!("{}", "[-] Found space in password. Ignoring...".red());
            return 1;
        } else {
            pass = &args[..];
        }

        println!("{}","[ ] Updating password...".yellow());
        dfile.update_pass(pass.to_string());
    } 

    // assuming something doesnt work or we dont get args, we just loop to try 
    // and save the user's file
    
    print!("{}", "[ ] Enter a password > ");
    std::io::stdout().flush().unwrap();
    let mut r = String::new();
    std::io::stdin().read_line(&mut r).expect("Failed to read STDIN");
    
    // try to save the file
    dfile.update_pass(r);
    0    
}

/// like `pass()`, but instead returns string of password. Useful for states 
/// before Datafile is initialized
fn get_pass() -> String {
    print!("{}", "[ ] Enter password > ".green());
    std::io::stdout().flush().unwrap();
    let mut r = String::new();
    std::io::stdin().read_line(&mut r).expect("Failed to read STDIN");
    let r = r.replace("\n", "");
    
    r
}

/// prints help info
fn help(_args: String, _dfile: &mut Datafile) -> u32 {
    // go through each command
    for comm in COMMS {
        println!("\t{}\t{}", comm.value, comm.help)
    }
    0
}

fn main() {
    // fetch our CLI arguments and parse them
    let matches = App::new("File-system to File")
                        .version("1.0")
                        .author("Bingo_Chado")
                        .about("Creates an encrypted representation of a collection of files")
                        .arg(Arg::with_name("datafile")
                            .short("d")
                            .long("datafile")
                            .value_name("FILE")
                            .takes_value(true)
                            .required(true)
                            .help("Path to the storage file I should access"))
                        .get_matches();

    let mut dfile: Datafile;
    let path = match matches.value_of("datafile") {
        Some(a) => a.to_string(),
        None => panic!("Failed to get argument value: no argument value provided")
    };

    // see if we are gonna try to make a new file or if we are working with a pre-existing one
    let aes_pass = get_pass();
    dfile = match Datafile::checked_new(path, aes_pass){
        Ok(a) => a,
        Err(e) => panic!("Failed to read data file: {}", e)
    };
    
    // begin our main interaction loop
    loop {
        let mut user_cmd = String::new();
        print!("{}", " > ".green());
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut user_cmd).unwrap(); 
        let user_cmd = user_cmd.replace("\n", "");

        let cmd: &str;
        let args: &str;
        if user_cmd.contains(" ") {
            let space_index = user_cmd.as_bytes().iter().position(|&r| r == b' ').unwrap();
            cmd = &user_cmd[..space_index];
            args = &user_cmd[space_index..]
        
        } else {
            cmd = &user_cmd[..];
            args = &"";
        }

        let mut ret_val = 0xffffffff;
        // figure out what command it is
        for comm in COMMS {
            if cmd == comm.value {
                let func = comm.command;
                ret_val = func(args.to_string(), &mut dfile);
            }
        } 

        // make sure the command actually went through
        if ret_val == 0xffffffff {
            println!("{}: {}", "[-] Unknown command. Type 'help' for a list of available commands".red(), cmd);
        }

    }
}
