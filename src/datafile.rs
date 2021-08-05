use std::io::Read;
use std::io::Write;
use std::str;

use openssl::symm::*;
use hmac_sha256::Hash;

const IV: &'static [u8] = b"1234567890ABCDEF";


/// The magic bytes that lead a decrypted file
const MAGIC_BYTES: &[u8; 16] = b"\x2b\xa4\x81\xab\x2b\xa4\x81\xab\x1b\x1b\x19\x0b\x56\xc2\xe7\xff";


/// Helper function to transform a password into a hashed format thats useful for AES
pub fn pass_to_hash(pass: String) -> [u8; 32] {
    Hash::hash(pass.as_bytes())
}



////////////////////////// DEFINITIONS /////////////////////////////////////
/// Enum that says if a file is currently stored in the Datafile or in its own buffer
enum StorageLocation {
    DatFile,
    OwnMem
}
/// Our struct that defines files in a datafile
pub struct EncFile {
    name: Vec<u8>,
    size: usize,
    offset: usize,
    fdat: Vec<u8>,
    location: StorageLocation
}

/// Our struct that defines a datafile
pub struct Datafile {
    file_data: Vec<u8>,
    aes_pass: [u8; 32],
    files: Vec<EncFile>
}


/////////////////////////// PARTIALEQ IMPL //////////////////////////////////
impl PartialEq for EncFile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && 
        self.offset == other.offset &&
        self.size == other.size 
    }
}



/////////////////////////// DISPLAY IMPL ////////////////////////////////////
/// implement print formatting for EncFile
impl std::fmt::Display for EncFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tmpbf = match str::from_utf8(&self.name) {
            Ok(a) => a,
            Err(e) => panic!("Unable to convert filename to string: {}", e)
        };
        write!(f, "{} ({} bytes)", tmpbf, self.size)
    }
}

/// implement print formatting for Datafile
impl std::fmt::Display for Datafile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dataset with {} files", self.files.len())
    }
}

////////////////////////// ENCFILE FUNCTIONS //////////////////////////
impl EncFile {
    #[allow(dead_code)]
    fn new(name: Vec<u8>, size: usize, offset: usize, load_self: Option<&Vec<u8>>) -> Self {
        // see if we are gonna load the file from
        let location: StorageLocation; 
        let fdat = match load_self {
            Some(a) => {
                let mut thing = vec![0u8;size];
                thing.copy_from_slice(&a[..]);
                location = StorageLocation::OwnMem;
                thing
            },
            None => {
                location = StorageLocation::DatFile;
                Vec::new()
            }
        };
        EncFile{name, size, offset, fdat, location}
    }

    /// serializes an EncFile for the datafile
    fn serialize(&self) -> Vec<u8> {
        let name_size = self.name.len();
        let mut data_buff: Vec<u8> = vec![0u8; name_size]; 

        // now that we have the buffer prepared, write the data to it
        data_buff.copy_from_slice(&self.name[..]);
        data_buff.push(0u8); // add in the null-term for reading it later
        for byte in self.size.to_ne_bytes() {
            data_buff.push(byte);
        }
        for byte in self.offset.to_ne_bytes() {
            data_buff.push(byte);
        }
        
        // return the serialized data
        data_buff
    }

    /// gets the file's name (cloned already)
    pub fn get_fname(&self) -> Vec<u8> {
        self.name.clone()
    }

    /// gets the file's size
    #[allow(dead_code)]
    pub fn get_fsize(&self) -> usize {
        self.size
    }

    /// gets the file's start offset
    #[allow(dead_code)]
    pub fn get_foffset(&self) -> usize {
        self.offset
    }

    /// loads the data of the file into memory
    #[allow(dead_code)]
    pub fn set_file_data(&mut self, path: String) -> Result<(), String> {
        // try to open the new file
        let mut f = match std::fs::File::open(path) {
            Ok(a) => a,
            Err(e) => return Err(e.to_string())
        };

        let mut data: Vec<u8> = Vec::new();
        
        // copy the file's data to the data vector
        let newfsize = match f.read_to_end(&mut data){
            Ok(a) => a,
            Err(e) => return Err(format!("Failed to read data: {}", e))
        };

        // update the variables we have in this file
        self.size = newfsize;
        self.fdat.clear();
        self.fdat.append(&mut data);
        self.update_location(StorageLocation::OwnMem);

        Ok(())
    }

    /// sets the direct contents of the file data
    fn set_file_vec(&mut self, dat: &mut Vec<u8>) -> Result<(), String> {
        // update the variables we have in this file
        self.fdat.clear();
        self.fdat.append(dat);

        // update the marker for data storage location 
        self.update_location(StorageLocation::OwnMem);

        Ok(())
    }

    fn get_fdat(&self) -> Result<&Vec<u8>, String> {
        match &self.location {
            StorageLocation::DatFile => return Err("Data is stored in Datafile".to_string()),
            StorageLocation::OwnMem => return Ok(&self.fdat)
        }
    }

    fn update_location(&mut self, loc: StorageLocation) {
        self.location = loc
    }

}

/////////////////////////// DATAFILE FUNCTIONS //////////////////////////
impl Datafile {
    /// creates a new Datafile, and creates a new file for it
    pub fn setup_new(aes_pass: String, filepath: String) -> Result<Self, String> {
        let pass = pass_to_hash(aes_pass.clone());
    
        let mut f = match std::fs::File::create(filepath.clone()) {
            Ok(a) => a,
            Err(e) => panic!("Failed to create file: {}",e)
        };

        // set up our checker vector
        let mut fvec = vec![0u8; MAGIC_BYTES.len()];
        fvec.copy_from_slice(MAGIC_BYTES);
        for _ in 0..4 {
            fvec.push(0);
        }
        println!("Length of data: {}", fvec.len());

        // set up the cipher and try to encrypt
        let t = Cipher::aes_256_cbc();
        let cyp = encrypt(t, &pass, Some(IV), &fvec[..]).unwrap();

        let mut written = match f.write(&cyp[0..cyp.len()]) {
            Ok(a) => a,
            Err(e) => panic!("{}", e)
        };

        while written != cyp.len() {
            written += match f.write(&cyp[written..cyp.len()]) {
                Ok(a) => a,
                Err(e) => panic!("{}", e)
            };
        }
    
        println!("Successfully created new file");

        Datafile::checked_new(filepath, aes_pass)
    }

    /// creates a new Datafile, checking to make sure it can successfully decrypt the data
    pub fn checked_new(filepath: String, aes_pass: String) -> Result<Self, String> {
        let mut file_handle = match std::fs::File::open(filepath) {
            Ok(a) => a,
            Err(e) => panic!("Failed to open file: {}", e)
        };
        // try to read the file's data
        let fsize = file_handle.metadata().unwrap().len();
        let mut data: Vec<u8> = Vec::new();
        // read the data from the file into the vector
        let datread = match file_handle.read_to_end(&mut data) {
            Ok(a) => a,
            Err(e) => panic!("Failed to read file data ({})", e)
        };
        // make sure metadata and read data lengths match
        assert_eq!(datread, fsize as usize, "Data lengths mismatch");
        
        
    
        let pass = pass_to_hash(aes_pass.clone());
        
        let t = Cipher::aes_256_cbc();
        let out = decrypt(t, &pass, Some(IV), &data[..]).unwrap();
        
        // assert that the data begins with the magic bytes 
        if &out[..16] != MAGIC_BYTES{ 
            println!("Failed");
            println!("Current: {:?}", out);
            println!("Magic: {:?}", MAGIC_BYTES);
            return Err("Magic bytes not found".to_string());
        };
        
        println!("Decryption successful");
        
        let mut df = Datafile::new(out, pass_to_hash(aes_pass));
        df.parse_filetable()?;

        Ok(df)
    }

    /// creates a new Datafile
    fn new(file_data: Vec<u8>, aes_pass: [u8; 32]) -> Self {
        let files: Vec<EncFile> = Vec::new();
        Datafile{file_data, aes_pass, files}
    }

    /// loads a new database from a file
    pub fn load_new(&mut self, path: String, passwd: String) -> Result<(), String> {
        let mut file_handle = match std::fs::File::open(path) {
            Ok(a) => a,
            Err(e) => panic!("Failed to open file: {}", e)
        };
        // try to read the file's data
        let fsize = file_handle.metadata().unwrap().len();
        let mut data: Vec<u8> = Vec::new();
        // read the data from the file into the vector
        let datread = match file_handle.read_to_end(&mut data) {
            Ok(a) => a,
            Err(e) => panic!("Failed to read file data ({})", e)
        };
        // make sure metadata and read data lengths match
        assert_eq!(datread, fsize as usize, "Data lengths mismatch");
        
        
    
        let pass = pass_to_hash(passwd.clone());
        
        let t = Cipher::aes_256_cbc();
        let mut out = decrypt(t, &pass, Some(IV), &data[..]).unwrap();
        
        // assert that the data begins with the magic bytes 
        if &out[..16] != MAGIC_BYTES{ 
            println!("Failed");
            println!("Current: {:?}", out);
            println!("Magic: {:?}", MAGIC_BYTES);
            return Err("Magic bytes not found".to_string());
        };
        
        println!("Decryption successful");
        
        self.file_data.clear();
        self.file_data.append(&mut out);
        self.aes_pass = pass_to_hash(passwd.clone());
        self.files.clear();

        self.parse_filetable()?;

        Ok(())

    }


    /// writes the data contained in self to a file
    pub fn save(&mut self, path: String) -> Result<(), String> {
        let write_buffer = match self.get_file_content() {
            Ok(a) => a,
            Err(e) => return Err(e)
        };
        // encrypt the data
        let t = Cipher::aes_256_cbc();
        let cyp = encrypt(t, &self.aes_pass, Some(IV), &write_buffer[..]).unwrap();

        // try to open the file for writing
        let mut f = match std::fs::File::create(path) {
            Ok(a) => a,
            Err(e) => return Err(e.to_string())
        };

        // write the data to the file
        let mut written = match f.write(&cyp[0..cyp.len()]) {
            Ok(a) => a,
            Err(e) => panic!("{}", e)
        };

        while written != cyp.len() {
            written += match f.write(&cyp[written..cyp.len()]) {
                Ok(a) => a,
                Err(e) => panic!("{}", e)
            };
        }

        Ok(())
    }


    /// attempt to parse the file table
    pub fn parse_filetable(&mut self) -> Result<(), String> {
        // get the number of available files from the data
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.file_data[16..20]);
        let num_files = u32::from_ne_bytes(bytes);
        println!("Number of files in the store: {}", num_files);

        // now that we know the number of files in the thing, try to read the 
        // file's information from the table
        let mut read_bytes = 16+8; // we have read the magic bytes and the number of files
        for _ in 0..num_files {
            let mut name_vec: Vec<u8> = Vec::new();
            
            // read until we find the nullterm of the file's name
            while self.file_data[read_bytes] != 0 {
                name_vec.push(self.file_data[read_bytes]);
                read_bytes += 1;
            }
            
            // we are now at the end of the string, so add one and find the file's size and offset values
            read_bytes += 1;
            let mut size_buf = [0u8; 8];
            let mut offset_buf = [0u8; 8];
            size_buf.copy_from_slice(&self.file_data[read_bytes..read_bytes+8]);
            offset_buf.copy_from_slice(&self.file_data[read_bytes+8..read_bytes+16]);
            let size = usize::from_ne_bytes(size_buf);
            let offset = usize::from_ne_bytes(offset_buf);

            // create a new file and append it to the structure
            self.files.push(EncFile::new(name_vec, size, offset, None));

            // update our read pointer
            read_bytes += 16;
        }

        Ok(())
    } 

    /// attempts to add a file to the store
    pub fn add_file(&mut self, name: Vec<u8>, path: String) -> Result<(), String> {
        // calculate the offset the file will have
        let last_bytes = match self.files.last() {
            Some(a) => a.offset + a.size, // find the offset of last file and add its size
            None => 0 // there are no files, so offset is default
        };

        // try to read the data into a vec
        let mut f = match std::fs::File::open(path) {
            Ok(a) => a,
            Err(e) => return Err(e.to_string())
        };
        let fsize = f.metadata().unwrap().len();
        let mut dvec: Vec<u8> = Vec::new();
        match f.read_to_end(&mut dvec) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string())
        };

        // try to push the file to the datafile and update its data content
        self.files.push(EncFile::new(name, fsize as usize, last_bytes, Some(&dvec)));
        Ok(())
    }

    /// returns the number of files stored in the structure
    pub fn num_files(&self) -> usize {
        self.files.len()
    }

    /// returns a vector of all parsed EncFiles
    pub fn files(&self) -> &Vec<EncFile> {
        &self.files
    }

    /// returns a mutable vector of all parsed EncFiles 
    pub fn files_mut(&mut self) -> &mut Vec<EncFile> {
        &mut self.files
    }

    /// dumps the decrypted data from the database to a file
    pub fn dump_self(&mut self) {
        let mut f = match std::fs::File::create("Dump.bin") {
            Ok(a) => a,
            Err(e) => {
                println!("Failed to open file: {}", e);
                return;
            }
        };

        f.write(&self.file_data[..]).unwrap();
    }

    /// returns the serialized content of the database
    fn get_file_content(&mut self) -> Result<Vec<u8>, String> {
        let mut write_buffer: Vec<u8> = Vec::new();

        // write the magic data
        for byte in MAGIC_BYTES {
            write_buffer.push(*byte);
        }
        
        // write the number of files available (as u32)
        for byte in self.num_files().to_ne_bytes() {
            write_buffer.push(byte);
        }

        let mut offset_ctr = self.get_table_size() + 24;

        // calculate and update the offsets of each file, as well as write the data
        for encf in self.files.iter_mut() {
            // Make sure the EncFile has its data stored locally 
            match &encf.location {
                StorageLocation::DatFile => {
                    let mut  dat = vec![0u8; encf.size];
                    let data = &self.file_data[encf.offset..encf.offset+encf.size];
                    dat.copy_from_slice(data);
                    match encf.set_file_vec(&mut dat) {
                        Ok(_) => (),
                        Err(e) => return Err(e)
                    };
                },
                StorageLocation::OwnMem => ()
            };

            // update our counters
            encf.offset = offset_ctr;
            offset_ctr += encf.size;
            

            // write the serialized table entry
            for byte in encf.serialize() {
                write_buffer.push(byte);
            }
        }

        // now write all the file's data sequentially
        for encf in self.files.iter() {
            // write the data
            let fdat = match encf.get_fdat() {
                Ok(a) => a,
                Err(e) => return Err(e)
            };
            for byte in fdat.iter() {
                write_buffer.push(*byte);
            }
        }

        Ok(write_buffer)

    }

    /// returns the size of the table structure as it stands right now
    fn get_table_size(&self) -> usize {
        let mut size = 0;
        // loop over each file and get its serialized size
        for file in self.files.iter() {
            size += file.serialize().len();
        }

        // return the size
        size
    }

    /// updates the AES passphrase for the database
    pub fn update_pass(&mut self, pass: String) {
        self.aes_pass = pass_to_hash(pass)
    }

    /// saves the decoded EncFile to a path 
    pub fn save_to_file(&self, file: &EncFile, path: String) -> Result<(), String> {
        let mut file_handle = match std::fs::File::create(path) {
            Ok(a) => a,
            Err(e) => panic!("Failed to open file: {}", e)
        };
        let size = file.size;

        let mut retvec: Vec<u8> = vec![0u8; size];
                
        let data = match file.get_fdat() {
            Ok(a) => a,
            Err(_) => { 
                // we know that the data is not stored in the file structure
                // itself, so we manually fetch it internally
                let offset = file.offset;
                retvec.copy_from_slice(&self.file_data[offset..offset+size]);
                &retvec
            }
        };

        file_handle.write_all(data).unwrap();
        
        Ok(())
    }

    /// removes an EncFile by reference
    pub fn remove_file(&mut self, file: &EncFile) {
        // find the index
        let index = self.files.iter().position(|x| x == file).unwrap();
        self.files.remove(index);
    }

    /// removes an EncFile by index
    pub fn remove_file_idx(&mut self, file_index: usize) {
        self.files.remove(file_index);
    }

}
