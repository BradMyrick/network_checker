use std::{thread, time::Duration};
use sysinfo::{NetworkExt, System, SystemExt};
use colored::Colorize;
use std::io::{stdout, Write};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::new_all();
    
    // First measurement to establish baseline
    system.refresh_all();
    let mut prev_received = HashMap::new();
    let mut prev_transmitted = HashMap::new();
    
    // Store initial values
    let networks = system.networks();
    for (interface_name, data) in networks {
        prev_received.insert(interface_name.to_string(), data.total_received());
        prev_transmitted.insert(interface_name.to_string(), data.total_transmitted());
    }
    
    // Wait before first measurement
    thread::sleep(Duration::from_secs(1));

    ctrlc::set_handler(move || {
        println!("\n{}", "Exiting network monitor...".yellow());
        std::process::exit(0);
    })?;

    loop {
        print!("\x1B[2J\x1B[1;1H");
        
        system.refresh_all();
        system.refresh_networks_list();

        println!("{}", "=== Network Interfaces ===".green().bold());
        println!("{}", format!("Updated at: {}", chrono::Local::now().format("%H:%M:%S")).blue());
        
        let networks = system.networks();
        
        for (interface_name, data) in networks {
            let prev_rx = prev_received.get(interface_name).unwrap_or(&0);
            let prev_tx = prev_transmitted.get(interface_name).unwrap_or(&0);
            
            let bytes_recv = data.total_received().saturating_sub(*prev_rx);
            let bytes_sent = data.total_transmitted().saturating_sub(*prev_tx);
            
            println!("\n{}", format!("Interface: {}", interface_name).cyan());
            println!("  Received:  {} bytes/s", format_bytes(bytes_recv).yellow());
            println!("  Transmitted: {} bytes/s", format_bytes(bytes_sent).yellow());
            
            // Update previous values
            prev_received.insert(interface_name.to_string(), data.total_received());
            prev_transmitted.insert(interface_name.to_string(), data.total_transmitted());
        }

        // Get interface details
        match if_addrs::get_if_addrs() {
            Ok(interfaces) => {
                println!("\n{}", "=== Interface Details ===".green().bold());
                for iface in interfaces {
                    println!("\n{}", format!("Interface: {}", iface.name).cyan());
                    println!("  IP Address: {}", iface.addr.ip().to_string().yellow());
                }
            }
            Err(e) => {
                eprintln!("{}: {}", "Error getting interface details".red().bold(), e);
            }
        }

        stdout().flush()?;
        thread::sleep(Duration::from_secs(1));
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}