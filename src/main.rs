use colored::*;
use serde_json::Value;
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines().map_while(Result::ok) {
        println!("{}", format_line(&line));
    }
}

fn format_line(line: &str) -> String {
    serde_json::from_str::<Value>(line)
        .ok()
        .and_then(|json| format_json_log(&json))
        .unwrap_or_else(|| colorize_plain_text(line))
}

fn format_json_log(json: &Value) -> Option<String> {
    let time = json
        .get("time")
        .and_then(Value::as_str)
        .and_then(extract_time)
        .map(|t| t.dimmed().to_string())
        .unwrap_or_default();

    let level = json
        .get("level")
        .and_then(Value::as_str)
        .map(colorize_level)
        .unwrap_or_default();

    let msg = json
        .get("msg")
        .or_else(|| json.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("");

    if msg == "START" || msg == "END" {
        return None;
    }

    // Extract key fields
    let method = json.get("method").and_then(Value::as_str);
    let path = json.get("path").and_then(Value::as_str);
    let status = json
        .get("statusCode")
        .and_then(Value::as_u64)
        .map(|s| s as u16);

    // Format based on what fields we have
    let formatted = match (method, path, status) {
        (Some(method), Some(path), Some(status)) => {
            // HTTP request log
            format!(
                "{} {} {} {} {}",
                time,
                level,
                format_method(method),
                path,
                format_status(status)
            )
        }
        _ => {
            // Regular log message
            format!("{time} {level} {msg}")
        }
    };

    Some(formatted.trim().to_string())
}

fn extract_time(time_str: &str) -> Option<String> {
    time_str
        .split('T')
        .nth(1)
        .and_then(|t| t.split('.').next())
        .map(String::from)
}

fn colorize_level(level: &str) -> String {
    match level.to_uppercase().as_str() {
        "ERROR" | "FATAL" => "ERROR".red().bold().to_string(),
        "WARN" | "WARNING" => "WARN ".yellow().to_string(),
        "INFO" => "INFO ".cyan().to_string(),
        "DEBUG" => "DEBUG".dimmed().to_string(),
        _ => level.to_string(),
    }
}

fn format_method(method: &str) -> String {
    let colored = match method {
        "GET" => method.blue(),
        "POST" => method.green(),
        "PUT" => method.yellow(),
        "DELETE" => method.red(),
        "PATCH" => method.magenta(),
        _ => method.normal(),
    };
    format!("{colored:6}")
}

fn format_status(status: u16) -> String {
    let status_str = status.to_string();
    match status {
        200..=299 => status_str.green().bold().to_string(),
        300..=399 => status_str.cyan().to_string(),
        400..=499 => status_str.yellow().bold().to_string(),
        500..=599 => status_str.red().bold().to_string(),
        _ => status_str.normal().to_string(),
    }
}

fn colorize_plain_text(line: &str) -> String {
    if line.contains("error") || line.contains("Error") || line.contains("ERROR") {
        line.red().to_string()
    } else if line.contains("warn") || line.contains("Warn") || line.contains("WARN") {
        line.yellow().to_string()
    } else {
        line.to_string()
    }
}
