use clap::{Arg, App};

mod datafile;


fn main() {
    // fetch our CLI arguments and parse them
    let matches = App::new("File-system to File")
                        .version("1.0")
                        .author("Bingo_Chado")
                        .about("Creates an encrypted representation of a collection of files")
                        .arg(Arg::with_name("new")
                            .short("n")
                            .long("new")
                            .value_name("FILE")
                            .takes_value(true)
                            .help("Creates a new storage file"))
                        .arg(Arg::with_name("datafile")
                            .short("d")
                            .long("datafile")
                            .value_name("FILE")
                            .takes_value(true)
                            .help("Path to the storage file I should access"))
                        .get_matches();

    let mut dfile: datafile::Datafile;

    // see if we are gonna try to make a new file or if we are working with a pre-existing one
    if matches.is_present("new") {
        // make a new file 
        let aes_pass = "123456".to_string();
        let path = "test.df".to_string();
        dfile = match datafile::Datafile::setup_new(aes_pass, path) {
            Ok(a) => a,
            Err(e) => panic!("Failed to create new data file: {}", e)
        };
    } else if matches.is_present("datafile") {
        // handle the opening of a new datafile
        let aes_pass = "123456".to_string();
        dfile = match datafile::Datafile::checked_new("test.df".to_string(), aes_pass){
            Ok(a) => a,
            Err(e) => panic!("Failed to read data file: {}", e)
        };
    } else {
        println!("No arguments provided! Use --help for more info");
        println!("{}", matches.usage());
        std::process::exit(1);
    }

    
    // try to parse the table
    match dfile.parse_filetable() {
        Ok(_) => (),
        Err(e) => panic!("Failed to parse filetable: {}", e)
    };

    // try to add a file to the datafile
    let name = "notes.md".to_string();
    let path = "notes.md".to_string();
    
    match dfile.add_file(name.into_bytes(), path) {
        Ok(_) => (),
        Err(e) => println!("Failed to add file: {}", e)
    };
    println!("Successfully added file to store. Number of files: {}", dfile.num_files());

    // dump the decrypted file
    dfile.dump_self();
    println!("Dumped self");

    // try to save the datafile
    match dfile.save("test.df".to_string()) {
        Ok(_) => (),
        Err(e) => println!("Failed to save data: {}", e)
    };

    println!("Save complete");
}
