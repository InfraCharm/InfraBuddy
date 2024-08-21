// Developed by InfraCharm LLC

use reqwest::blocking::Client;
use sysinfo::{System, SystemExt, ProcessorExt, NetworkExt};
use humantime::parse_duration;
use std::thread::sleep;
use std::process::{Command, Stdio};
use std::fs;
use serde_json::json;
use tokio::time::{sleep as tokio_sleep, Duration};
use std::io::{BufRead, BufReader};
use std::str;
use std::collections::HashMap;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use core::num::ParseIntError;

#[derive(Debug, Deserialize)]
struct Config {
    webhook_url: String,
    embed_title: String,
    embed_color: String,
    update_interval: String,
    optional_message: String,
    user_tags: Vec<String>,
    show_memory: bool,
    memory_in_mb: bool,
    show_cpu: bool,
    show_network_usage: bool,
    network_interfaces: HashSet<String>,
    optional_message_enabled: bool,
    user_tags_enabled: bool,
    update_previous_message: bool,
    message_id: Option<String>,
    show_disk_usage: bool,
    disk_drives: HashSet<String>,
    disk_names: HashMap<String, String>,
    ssh_alerts: SshAlertsConfig,
}

#[derive(Debug, Deserialize)]
struct SshAlertsConfig {
    enabled: bool,
    log_path: String,
    ssh_alert_webhook_url: String,
}

fn load_config() -> Config {
    let config_content = fs::read_to_string("config.toml").expect("Failed to read config.toml");
    toml::from_str(&config_content).expect("Failed to parse config.toml")
}

impl Config {
    fn get_embed_color(&self) -> Result<u32, ParseIntError> {
        u32::from_str_radix(&self.embed_color.trim_start_matches('#'), 16)
    }
}

fn get_disk_usage(config: &Config) -> HashMap<String, (f64, f64)> {
    let mut disk_usage_map = HashMap::new();

    for drive in &config.disk_drives {
        let output = Command::new("df")
            .arg("-B1")
            .arg("-P")
            .arg(&drive)
            .output()
            .expect("Failed to execute df command");

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut lines = output_str.lines();

            lines.next();

            if let Some(line) = lines.next() {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 4 {
                    let used_kb = fields[2].parse::<f64>().unwrap_or(0.0) / 1024.0 / 1024.0 / 1024.0; // Convert KB to GB
                    let available_kb = fields[3].parse::<f64>().unwrap_or(0.0) / 1024.0 / 1024.0 / 1024.0; // Convert KB to GB
                    disk_usage_map.insert(drive.clone(), (used_kb, available_kb));
                }
            }
        }
    }

    disk_usage_map
}

fn bytes_to_gb(bytes: f64) -> f64 {
    bytes as f64 / (1024.0) + 428.30
}

fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0) + 0.47
}

fn bytes_to_mbps(bytes: u64) -> f64 {
    bytes as f64 * 8.0 / (1024.0 * 1024.0)
}

fn send_embed(config: &Config, system: &System) {
    let color = config.get_embed_color().unwrap_or_default();

    let cpu_usage = if config.show_cpu {
        Some(system.get_global_processor_info().get_cpu_usage())
    } else {
        None
    };

    let used_memory_bytes = if config.show_memory {
        system.get_used_memory()
    } else {
        0
    };

    let used_memory = if config.memory_in_mb {
        bytes_to_gb(used_memory_bytes as f64)
    } else {
        bytes_to_mb(used_memory_bytes)
    };

    let network_usage = if config.show_network_usage {
        let mut network_usage_str = String::new();
        for (name, interface) in system.get_networks() {
            if config.network_interfaces.is_empty() || config.network_interfaces.contains(name.as_str()) {
                let input_traffic = interface.get_received();
                let output_traffic = interface.get_transmitted();

                let input_mbps = bytes_to_mbps(input_traffic);
                let output_mbps = bytes_to_mbps(output_traffic);

                network_usage_str.push_str(&format!(
                    "{}: In {:.2} Mbps | Out {:.2} Mbps\n",
                    name, input_mbps, output_mbps
                ));
            }
        }
        Some(network_usage_str)
    } else {
        None
    };

    let disk_usage_map = if config.show_disk_usage {
        get_disk_usage(&config)
    } else {
        HashMap::new()
    };

    let mut payload = json!({
        "embeds": [{
            "title": &config.embed_title,
            "fields": [
                {"name": "CPU Usage", "value": cpu_usage.map(|usage| format!("{:.2}%", usage)).unwrap_or_default(), "inline": true},
                {"name": "Used Memory", "value": if config.show_memory { format!("{:.2} {}", used_memory, if config.memory_in_mb { "MB" } else { "GB" }) } else { "".to_string() }, "inline": true},
                {"name": "Network Usage", "value": network_usage.unwrap_or_default(), "inline": false},
            ],
            "color": color,
        }],
        "footer": {
            "text": format!("{} {}", config.optional_message, config.user_tags.join(" ")),
        },
    });

    if config.user_tags_enabled {
        payload["embeds"][0]["fields"].as_array_mut().unwrap().push(json!({
            "name": "User Tags",
            "value": config.user_tags.iter().map(|tag| format!("<@{}>", tag)).collect::<Vec<_>>().join(" "),
            "inline": true,
        }));
    }

    if config.optional_message_enabled {
        payload["embeds"][0]["footer"]["text"] = serde_json::json!(config.optional_message);
    }

    for (drive, (used, available)) in disk_usage_map {
        let friendly_name = config.disk_names.get(&drive).unwrap_or(&drive);
        payload["embeds"][0]["fields"].as_array_mut().unwrap().push(json!({
            "name": format!("Disk Usage ({})", friendly_name),
            "value": format!("Used: {:.2} GB\nAvailable: {:.2} GB", used, available),
            "inline": true,
        }));
    }

    let client = reqwest::blocking::Client::new();
    let response = if config.update_previous_message {
        if let Some(id) = &config.message_id {
            client.patch(&format!("{}/messages/{}", &config.webhook_url, id))
        } else {
            client.post(&config.webhook_url)
        }
    } else {
        client.post(&config.webhook_url)
    };

    let response = response
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&payload).expect("Failed to serialize JSON"))
        .send();

    match response {
        Ok(res) => {
            if res.status().is_success() {
                println!("Embed sent successfully!");
            } else {
                println!("Error sending embed. Status: {:?}", res.status());
                let body = res.text().unwrap_or("No response body".to_string());
                println!("Response body: {}", body);
            }
        }
        Err(err) => {
            println!("Error sending embed: {:?}", err);
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Arc::new(load_config());

    let main_loop_interval = parse_duration(&config.update_interval)
        .expect("Failed to parse update interval from configuration");

    let config_main_loop = Arc::clone(&config);

    tokio::spawn(async move {
        let mut system = System::new_all();
        loop {
            system.refresh_all();

            send_embed(&config_main_loop, &system);

            tokio_sleep(main_loop_interval).await;
        }
    });

    let last_login_details = Arc::new(Mutex::new((None, 0)));
    monitor_ssh_logins(&config, last_login_details);
    loop {
        tokio::time::interval(Duration::from_secs(1)).tick().await;
    }
}

fn get_hwid() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("dmidecode | grep -w UUID | sed \"s/^.UUID\\: //g\"")
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute dmidecode command");
    let hwid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() {
        eprintln!("Error running dmidecode command: {:?}", output.status);
    }

    hwid
}

fn monitor_ssh_logins(config: &Arc<Config>, last_login_details: Arc<Mutex<(Option<SshLoginDetails>, u64)>>) {
    if (!config.ssh_alerts.enabled) {
        return;
    }

    let log_path = &config.ssh_alerts.log_path;

    let child = Command::new("sh")
        .arg("-c")
        .arg(format!("tail -F {}", log_path))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute tail command");

    let stdout = child.stdout.expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("Accepted password for") || line.contains("Accepted publickey for") {
                let details = parse_ssh_login_details(&line);
                if let Some(details) = details {
                    let mut last_login = last_login_details.lock().unwrap();
                    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    if *last_login != (Some(details.clone()), current_time) {
                        if current_time - last_login.1 >= 2 {
                            send_ssh_login_embed(&config, details.clone());
                            *last_login = (Some(details), current_time);
                        }
                    }
                }
            }
        }
    }
}

fn parse_ssh_login_details(log_line: &str) -> Option<SshLoginDetails> {
    let parts: Vec<&str> = log_line.split_whitespace().collect();
    if parts.len() >= 11 {
        let time = format!("{} {} {}", parts[0], parts[1], parts[2]);
        let user = parts[8].to_string();
        let ip = parts[10].to_string();
        Some(SshLoginDetails { user, ip, time })
    } else {
        None
    }
}

#[derive(Clone, PartialEq, Eq)]
struct SshLoginDetails {
    user: String,
    ip: String,
    time: String,
}

fn send_ssh_login_embed(config: &Arc<Config>, details: SshLoginDetails) {
    let color = config.get_embed_color().unwrap_or_default();

    let payload = json!({
        "embeds": [{
            "title": &config.embed_title,
            "fields": [
                {"name": "User", "value": details.user, "inline": true},
                {"name": "IP Address", "value": details.ip, "inline": true},
                {"name": "Login Time", "value": details.time, "inline": true},
            ],
            "color": color,
        }],
    });

    let client = reqwest::blocking::Client::new();
    let response = client.post(&config.ssh_alerts.ssh_alert_webhook_url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&payload).expect("Failed to serialize JSON"))
        .send();

    match response {
        Ok(res) => {
            if res.status().is_success() {
                println!("SSH login embed sent successfully!");
            } else {
                println!("Error sending SSH login embed. Status: {:?}", res.status());
                let body = res.text().unwrap_or("No response body".to_string());
                println!("Response body: {}", body);
            }
        }
        Err(err) => {
            println!("Error sending SSH login embed: {:?}", err);
        }
    }
}
