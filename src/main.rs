use clap::{Arg, App};
use colored::Colorize;
use rpassword::prompt_password_stdout;
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
            command: remove
        },
        Command{
            value: "fetch",
            help: "Fetches a file from the current state and saves it to the filesystem",
            command: fetch
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
fn load(args: String, dfile: &mut Datafile) -> u32 {
    let passwd = get_pass();
        
    if !args.is_empty() {
        let path: &str;
        // see if we can save to the first path provided by the arguments
        if args.contains(" ") {
            let space_index = args.as_bytes().iter().position(|&r| r == b' ').unwrap();
            path = &args[..space_index];
        } else {
            path = &args[..];
        }

        println!("{}: {}","[ ] Opening database file".yellow(), path);
        match dfile.load_new(path.to_string(), passwd.clone()) {
            Ok(_) => {
                println!("{}", "[+] Success!".green());
                return 0
            },
            Err(e) => {
                println!("{}: {}", "[-] Failed to save file".red(), e);
            }
        };
    } 

    // assuming something doesnt work or we dont get args, we just loop to try 
    // and save the user's file
    loop {
        let passwd_clone = passwd.clone();
        print!("{}", "[ ] Enter path to the database > ");
        std::io::stdout().flush().unwrap();
    
        let mut path = String::new();
        std::io::stdin().read_line(&mut path).expect("Failed to read STDIN");
        let path = path.replace("\n", "");
        
        // try to save the file
        match dfile.load_new(path, passwd_clone) {
            Ok(_) => {
                println!("{}", "[+] Success!".green());
                return 0
            },
            Err(e) => println!("{}: {}", "[-] Failed to save file".red(), e)
        };
    }

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
        let r = r.replace("\n", "");
        
        // try to save the file
        match dfile.save(r) {
            Ok(_) => {
                println!("{}", "[+] Success!".green());
                return 0
            },
            Err(e) => println!("{}: {}", "[-] Failed to save file".red(), e)
        };
    }
}

/// adds a file
fn add(args: String, dfile: &mut Datafile) -> u32 {
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

        println!("{}: {}","[ ] Adding file".yellow(), path);
        let path = std::path::Path::new(path);
        let name = match path.file_name() {
            Some(a) => match a.to_os_string().into_string(){
                Ok(b) => b.into_bytes(),
                Err(_) => {
                    println!("{}", "[-] Failed to parse path".red());
                    return 1
                }
            },
            None => {
                println!("{}", "[-] Failed to parse path".red());
                return 1
            }
        };

        let path = match path.to_str(){
            Some(a) => a.to_string(),
            None => {
                println!("{}", "[-] Failed to parse path".red());
                return 1
            }
        };

        match dfile.add_file(name, path) {
            Ok(_) => return 0,
            Err(e) => println!("{}: {}", "[-] Failed to save file".red(), e)
        };
    } 

    // assuming something doesnt work or we dont get args, we just loop to try 
    // and save the user's file
    loop {
        print!("{}", "[ ] Enter path to new file > ");
        std::io::stdout().flush().unwrap();
    
        let mut r = String::new();
        std::io::stdin().read_line(&mut r).expect("Failed to read STDIN");
        let r = r.replace("\n", "");

        println!("{}: {}","[ ] Adding file".yellow(), r);
        let path = std::path::Path::new(&r);
        let name = match path.file_name() {
            Some(a) => match a.to_os_string().into_string(){
                Ok(b) => b.into_bytes(),
                Err(_) => {
                    println!("{}", "[-] Failed to parse path".red());
                    return 1
                }
            },
            None => {
                println!("{}", "[-] Failed to parse path".red());
                return 1
            }
        };

        let path = match path.to_str(){
            Some(a) => a.to_string(),
            None => {
                println!("{}", "[-] Failed to parse path".red());
                return 1
            }
        };
        
        // try to save the file
        match dfile.add_file(name, path) {
            Ok(_) => {
                println!("{}", "[+] Success!".green());
                return 0
            },
            Err(e) => println!("{}: {}", "[-] Failed to save file".red(), e)
        };
    }
}

/// removes a file
fn remove(_args: String, dfile: &mut Datafile) -> u32 {
    loop {
        ls("".to_string(), dfile);

        print!("{}", "[ ] Enter file name > ");
        std::io::stdout().flush().unwrap();
    
        let mut r = String::new();
        std::io::stdin().read_line(&mut r).expect("Failed to read STDIN");
        let r = r.replace("\n", "");
        let fname = r.clone();
        let r = r.into_bytes();

        let mut idx: i32 = 0;
        let mut found_idx: i32 = -1;
        // loop over each file and see if the name is the same
        for file in dfile.files_mut().iter_mut() {
            
            if file.get_fname() == r {
                found_idx = idx;
                break;                
            }
            idx += 1;
        }

        // see if we found the file
        if found_idx != -1 {
            println!("{}: {}", "[ ] Removing file", fname.clone());
                
            dfile.remove_file_idx(found_idx as usize);
            println!("{}", "[+] Success!");
            return 0
            
        }

        // if we get here, we know that the user failed to input the correct name
        println!("{}: {}", "[-] No file by that name found".yellow(), fname);

    }
}

/// fetches a file and stores it wherever the user wants it to be stored
fn fetch(args: String, dfile: &mut Datafile) -> u32 {
    
    if !args.is_empty() {
    
        // Will work on the argument parsing part later... 
        // gonna be a bit complicated with 2 potential args...
        unimplemented!();
    
        let path: &str;
        // see if we can save to the first path provided by the arguments
        if args.contains(" ") {
            let space_index = args.as_bytes().iter().position(|&r| r == b' ').unwrap();
            path = &args[..space_index];
        } else {
            path = &args[..];
        }

        println!("{}: {}","[ ] Adding file".yellow(), path);
        let path = std::path::Path::new(path);
        let name = match path.file_name() {
            Some(a) => match a.to_os_string().into_string(){
                Ok(b) => b.into_bytes(),
                Err(_) => {
                    println!("{}", "[-] Failed to parse path".red());
                    return 1
                }
            },
            None => {
                println!("{}", "[-] Failed to parse path".red());
                return 1
            }
        };

        let path = match path.to_str(){
            Some(a) => a.to_string(),
            None => {
                println!("{}", "[-] Failed to parse path".red());
                return 1
            }
        };

        match dfile.add_file(name, path) {
            Ok(_) => return 0,
            Err(e) => println!("{}: {}", "[-] Failed to save file".red(), e)
        };
    } 

    // assuming something doesnt work or we dont get args, we just loop to try 
    // and save the user's file
    loop {
        ls("".to_string(), dfile);

        print!("{}", "[ ] Enter file name > ");
        std::io::stdout().flush().unwrap();
    
        let mut r = String::new();
        std::io::stdin().read_line(&mut r).expect("Failed to read STDIN");
        let r = r.replace("\n", "");
        let fname = r.clone();
        let r = r.into_bytes();

        // loop over each file and see if the name is the same
        for file in dfile.files() {
            
            if file.get_fname() == r {
                print!("{}", "[ ] Enter file path to save > ");
                std::io::stdout().flush().unwrap();
    
                let mut path = String::new();
                std::io::stdin().read_line(&mut path).expect("Failed to read STDIN");
                let path = path.replace("\n", "");

                println!("{}: {}","[ ] Saving to filesystem".yellow(), path);
                match dfile.save_to_file(file, path) {
                    Ok(_) => {
                        println!("{}", "[+] Success!");
                        return 0
                    },
                    Err(e) => {
                        println!("{}: {}", "[-] Failed to fetch file".red(), e);
                        return 1
                    }

                }
                
            }
        }

        // if we get here, we know that the user failed to input the correct name
        println!("{}: {}", "[-] No file by that name found".yellow(), fname);

    }   


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
    let r = r.replace("\n", "");
    
    // try to save the file
    dfile.update_pass(r);
    0    
}

/// like `pass()`, but instead returns string of password. Useful for states 
/// before Datafile is initialized
fn get_pass() -> String {
    prompt_password_stdout(
        &format!("{}", "[ ] Enter password > ".green())[..]
    ).unwrap()
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
                            .help("Path to the storage file I should access"))
                        .arg(Arg::with_name("new")
                            .short("n")
                            .long("new")
                            .value_name("FILE")
                            .takes_value(true)
                            .help("Creates a new archive with name FILE"))
                        .get_matches();

    let mut dfile: Datafile;
    let mut path: String;

    // see if we are gonna try to make a new file or if we are working with a pre-existing one
    if !matches.is_present("new") && !matches.is_present("datafile") {
        println!("[-] Missing arguments");
        println!("{}", matches.usage());
        std::process::exit(1);
    }

    // loop until the user has successfully decrypted the file
    loop {
        let aes_pass = get_pass();
        if matches.is_present("new") {
            match matches.value_of("new") {
                Some(a) => {
                    path = a.to_string();
                    match Datafile::setup_new(aes_pass, path) {
                        Ok(a) => {
                            dfile = a;
                            break;
                        },
                        Err(e) => println!("{}{}", "[-] Failed to create new data file: ".red(), e)
                    }
                },
                None => println!("{}", "[-] Failed to get argument value: no argument value provided".red())
            };
        } else {
            match matches.value_of("datafile") {
                Some(a) => {
                    path = a.to_string();
                    match Datafile::checked_new(path, aes_pass){
                        Ok(a) => {
                            dfile = a;
                            break;
                        },
                        Err(e) => println!("{}{}","[-] Failed to read data file: ".red(), e)
                    };
                },
                None => println!("{}", "[-] Failed to get argument value: no argument value provided".red())
            };
            
        }

    }
    
    
    
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
            args = &user_cmd[space_index+1..]
        
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
