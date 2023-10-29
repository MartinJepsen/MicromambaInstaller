use std::env::consts::{ARCH, OS};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use curl::easy::{Easy2, Handler, WriteError};

enum OperatingSystem {
    Windows,
    Macos,
    Linux,
}

impl OperatingSystem {
    /// Convert the enum variant to a `String` that matches the naming convention of Micromamba
    fn as_string(&self) -> String {
        match self {
            OperatingSystem::Windows => String::from("win"),
            OperatingSystem::Macos => String::from("osx"),
            OperatingSystem::Linux => String::from("linux"),
        }
    }
}

/// A handler that writes data to a file.
struct FileHandler {
    /// The file to write data to. This is done in append mode.
    file: File,
}

impl FileHandler {
    /// Creates a file at the path and returns a new FileHandler.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the file path
    ///
    pub fn new(path: &String) -> FileHandler {
        let as_path: &Path = Path::new(path);
        // Create parent directories if they do not exist yet
        let parent: &Path = as_path.parent().unwrap();
        if !parent.is_dir() {
            std::fs::create_dir_all(parent).unwrap();
        };
        // Create the executable file
        File::create(path).unwrap();

        FileHandler {
            file: File::options().append(true).open(path).unwrap(),  // Set write options for executable file
        }
    }
}

impl Handler for FileHandler {
    /// Implement the `write` method of the `Handler` trait.
    /// This appends the received bytes to `self.file`.
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        let bytes_received = data.len(); // Count the bytes received
        self.file.write_all(data).unwrap(); // Write the bytes to the file
        println!("Received {bytes_received} bytes.");
        Ok(bytes_received) // Must return the number of bytes that were passed to data
    }
}

struct MicromambaConfig {
    exe_path: String,
    init_shell: bool,
    root_prefix: String,
    shell: Option<String>,
}

impl MicromambaConfig {
    fn get_root_prefix() -> String {
        let mut path: String = String::from("~/micromamba/");
        println!("Micromamba root prefix? [{}]", path);
        let mut user_input: String = String::new();
        std::io::stdin().read_line(&mut user_input).unwrap();
        user_input = String::from(user_input.trim());
        if !(user_input.is_empty()) {
            path = user_input;
        };
        path
    }

    fn init_shell() -> bool {
        let mut answer = String::new();
        println!("Initialize micromamba (shell is chosen later)? ([y]/n)");
        std::io::stdin().read_line(&mut answer).unwrap();
        match answer.trim() {
            "" | "y" | "Y" | "yes" => true,
            "n" | "N" | "no" => false,
            _ => panic!("Invalid answer: {}", answer),
        }
    }
    fn ask_for_shell() -> String {
        println!(
            "Select the shell to initialize:\n\
            \n\
            Options are {{bash,cmd.exe,dash,fish,posix,powershell,tcsh,xonsh,zsh}}"
        );
        let mut user_input = String::new();
        std::io::stdin().read_line(&mut user_input).unwrap();
        String::from(user_input.trim())
    }

    fn get_bin_path(os: &OperatingSystem) -> String {
        let mut path = match os {
            OperatingSystem::Windows => String::from("~/micromamba/micromamba.exe"),
            OperatingSystem::Macos | OperatingSystem::Linux => String::from("~/.local/bin/micromamba"),
        };
        println!("Micromamba binary path? [{}]", path);
        let mut user_input: String = String::new();
        std::io::stdin().read_line(&mut user_input).unwrap();
        user_input = String::from(user_input.trim());
        if !(user_input.is_empty()) {
            path = user_input;
        };
        path
    }
    pub fn new(os: &OperatingSystem) -> MicromambaConfig {
        let root_prefix: String = MicromambaConfig::get_root_prefix();
        let init_shell: bool = MicromambaConfig::init_shell();
        let shell: Option<String> = if init_shell {
            Some(MicromambaConfig::ask_for_shell())
        } else {
            None
        };
        let exe_path: String = MicromambaConfig::get_bin_path(&os);
        
        MicromambaConfig {
            exe_path,
            init_shell,
            root_prefix,
            shell,
        }
    }
}

fn main() -> () {
    // Get the current OS as an OperatingSystem type
    let _os: OperatingSystem = match OS {
        "windows" => OperatingSystem::Windows,
        "linux" => OperatingSystem::Linux,
        "macos" => OperatingSystem::Macos,
        _ => panic!("Unsupported operating system: {OS}"),
    };
    // Initialize micromamba configuration
    let _config = MicromambaConfig::new(&_os);
    download_micromamba_exe(_os, &_config.exe_path).unwrap();
    if _config.init_shell {
        init_micromamba(&_config);
    }
}

fn download_micromamba_exe(os: OperatingSystem, exe_path: &String) -> Result<(), String> {
    // Determine what download URL we should query, based on OS and architecture
    let os_arch: String = determine_os_arch(&os);
    let url: String = format!(
        "https://github.com/mamba-org/micromamba-releases/releases/latest/download/micromamba-{}",
        os_arch
    );
    println!("Sending request to {}", &url);

    // Download the executable
    let mut easy = Easy2::new(FileHandler::new(&exe_path));
    easy.get(true).unwrap();
    easy.follow_location(true).unwrap();
    easy.url(&url).unwrap();
    easy.perform().unwrap();

    // Check the response code
    let response_code: u32 = easy.response_code().unwrap();
    match response_code {
        200 => Ok(()),
        _ => Err(format!("Got response code {}", response_code)),
    }
}

/// Determine the ${OS}-${ARCH} part of the GitHub download URL.
fn determine_os_arch(os: &OperatingSystem) -> String {
    let mut os_arch = os.as_string();
    os_arch += "-";

    os_arch += match ARCH {
        "x86_64" => "64",
        "arm" => "arm64",
        _ => panic!("Unsupported architecture {:?}", ARCH),
    };
    os_arch
}

fn init_micromamba(config: &MicromambaConfig) {
    println!("Initializing micromamba for current shell with");
    println!("{}", &config.exe_path);
    let shell: &String = match &config.shell {
        Some(i) => i,
        None => panic!("Tried shell initialization without a shell.")
    };
    let mut micromamba = Command::new(&config.exe_path);
    micromamba.arg("shell").arg("init").arg("--prefix").arg(&config.root_prefix).arg("--shell").arg(shell);
    micromamba.status().unwrap();
}
