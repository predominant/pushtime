use directories::BaseDirs;
use pushover::requests::message::{SendMessage, SendMessageResponse};
use pushover::API;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    time::Instant,
};

#[derive(Debug)]
struct PushoverConfig {
    token: String,
    user: String,
    api_url: Option<String>,
}

fn main() {
    let mut args = env::args().skip(1);
    let command = match args.next() {
        Some(cmd) => cmd,
        None => {
            eprintln!("No command was provided.");
            std::process::exit(1);
        }
    };

    // Read Pushover configuration
    let config = match find_pushover_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let command_args: Vec<String> = args.collect();
    let full_command = format!("{} {}", command, command_args.join(" "))
        .trim_end()
        .to_string();
    println!("Running command: {}", full_command);

    // Start the timer
    let start = Instant::now();

    // Execute the command
    let status = match execute_command(&command, &command_args) {
        Ok(status) => status,
        Err(e) => {
            eprintln!("Error executing command: {}", e);
            std::process::exit(1);
        }
    };

    // Calculate the elapsed time
    let duration = start.elapsed();

    // Print the result
    println!("Command executed: {}", full_command);
    println!("Execution time: {:?}", duration);
    println!("Exit status: {}", status);

    // Send the notification
    send_notification(&config, &full_command, status, duration)
        .expect("Failed to send notification");
}

fn execute_command(command: &str, args: &[String]) -> Result<ExitStatus, std::io::Error> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .expect("Failed to take stdout of child process");
    let stderr = child
        .stderr
        .take()
        .expect("Failed to take stderr of child process");

    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("{}", line),
                Err(e) => eprintln!("Error reading stdout: {}", e),
            }
        }
    });

    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("{}", line),
                Err(e) => eprintln!("Error reading stderr: {}", e),
            }
        }
    });

    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    child.wait()
}

fn find_pushover_config() -> Result<PushoverConfig, String> {
    let current_dir = std::env::current_dir().unwrap();
    for directory in current_dir.ancestors() {
        let config_path = directory.join(".pushtime");
        if config_path.exists() {
            return parse_pushover_config(config_path);
        }
    }

    if let Some(base_dirs) = BaseDirs::new() {
        let home_config_path = base_dirs.home_dir().join(".pushtime");
        if home_config_path.exists() {
            return parse_pushover_config(home_config_path);
        }
    }

    Err(
        "No .pushtime config found in the current or parent directories or home directory."
            .to_string(),
    )
}

fn parse_pushover_config(path: PathBuf) -> Result<PushoverConfig, String> {
    let contents = fs::read_to_string(&path)
        .map_err(|_| "Failed to read pushover configuration file.".to_string())?;

    let mut config_map = HashMap::new();
    for line in contents.lines() {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            config_map.insert(parts[0].trim(), parts[1].trim());
        }
    }

    let token = config_map
        .get("PUSHOVER_TOKEN")
        .ok_or_else(|| "PUSHOVER_TOKEN not found in config.".to_string())?
        .to_string();
    let user = config_map
        .get("PUSHOVER_USER")
        .ok_or_else(|| "PUSHOVER_USER not found in config.".to_string())?
        .to_string();
    let api_url = config_map.get("PUSHOVER_API").map(|url| url.to_string());

    if token.is_empty() || user.is_empty() {
        return Err("PUSHOVER_TOKEN or PUSHOVER_USER is empty in config.".to_string());
    }

    Ok(PushoverConfig {
        token,
        user,
        api_url,
    })
}
fn send_notification(
    config: &PushoverConfig,
    command: &str,
    status: ExitStatus,
    duration: std::time::Duration,
) -> Result<SendMessageResponse, String> {
    let default_api_url = "https://api.pushover.net";
    let api_url = config.api_url.as_deref().unwrap_or(default_api_url);

    let api = API::new().base_url(&api_url);

    let message_text = format!(
        "Command: {}\nFinished with exit code: {}\nExecution time: {:.2?}",
        command,
        status.code().unwrap_or(-1),
        duration
    );
    let message = SendMessage::new(&config.token, &config.user, &message_text);
    let response = api.send(&message);
    response.map_err(|e| format!("Failed to send notification: {:?}", e))
}
