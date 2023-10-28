
use std::env::consts::{ARCH, OS};
use std::fs::File;
use std::io::Write;

use curl::easy::{Easy2, Handler, WriteError};

/// A handler that writes data to a file.
struct FileHandler {
    /// The file to write data to. This is done in append mode.
    file: File
}

impl FileHandler {
    /// Creates a file at the path and returns a new FileHandler.
    /// 
    /// # Arguments
    /// 
    /// * `path` - A string slice representing the file path
    /// 
    pub fn new(path: &str) -> FileHandler {
        File::create(path).unwrap(); // Create the file
        FileHandler{file: File::options().append(true).open(path).unwrap()}  // Instantiate a new FileHandler with the given path
    }
}


impl Handler for FileHandler {
    /// Implement the `write` method of the `Handler` trait.
    /// This appends the received bytes to `self.file`.
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        let bytes_received = data.len();  // Count the bytes received
        self.file.write_all(data).unwrap();  // Write the bytes to the file
        println!("Received {bytes_received} bytes.");
        Ok(bytes_received)  // Must return the number of bytes that were passed to data
    }
}


fn main() -> () {

    // Determine what download URL we should query, based on OS and architecture
    let os_arch = determine_os_arch();  
    let url = format!("https://github.com/mamba-org/micromamba-releases/releases/latest/download/micromamba-{}", os_arch);
    println!("Sending request to {}", &url);


    let mut easy = Easy2::new(FileHandler::new("micromamba2.exe"));
    easy.get(true).unwrap();
    easy.follow_location(true).unwrap();
    easy.url(&url).unwrap();
    easy.perform().unwrap();

    println!("{:?}", easy.response_code().unwrap());
}

/// Determine the ${OS}-${ARCH} part of the GitHub download URL. 
fn determine_os_arch() -> String {

    let mut os_arch = String::new();

    if OS == "windows" {
        os_arch += "win";
    } else if OS == "linux" {
        os_arch += "linux";
    } else if OS == "macos" {
        os_arch += "osx";
    } else {
        panic!("Unsupported operating system {:?}", OS);
    }

    os_arch += "-";

    if ARCH == "x86_64" {
        os_arch += "64"
    } else if ARCH == "arm" {
        os_arch += "arm64"
    } else {
        panic!{"Unsupported architecture {:?}", ARCH};
    }

    os_arch
}
