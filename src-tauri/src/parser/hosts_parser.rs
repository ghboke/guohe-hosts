use regex::Regex;

use crate::error::AppError;
use crate::models::{HostEntry, HostGroup};

/// Parse hosts file content into groups.
///
/// Group markers use the format:
/// ```text
/// # @group Group Name
/// ...entries...
/// # @end
/// ```
///
/// Lines outside any group marker go into a "System" default group.
pub fn parse_hosts_content(content: &str) -> Result<Vec<HostGroup>, AppError> {
    let group_start_re =
        Regex::new(r"^#\s*@group\s+(.+)$").map_err(|e| AppError::ParseError(e.to_string()))?;
    let group_end_re =
        Regex::new(r"^#\s*@end\s*$").map_err(|e| AppError::ParseError(e.to_string()))?;

    let mut groups: Vec<HostGroup> = Vec::new();
    let mut system_group = HostGroup::new("System".to_string());
    let mut current_group: Option<HostGroup> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(caps) = group_start_re.captures(trimmed) {
            // If we were in a group, push it first
            if let Some(g) = current_group.take() {
                groups.push(g);
            }
            let name = caps.get(1).unwrap().as_str().trim().to_string();
            current_group = Some(HostGroup::new(name));
            continue;
        }

        if group_end_re.is_match(trimmed) {
            if let Some(g) = current_group.take() {
                groups.push(g);
            }
            continue;
        }

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        let entry = parse_line(trimmed);
        match current_group.as_mut() {
            Some(g) => {
                if let Some(e) = entry {
                    g.entries.push(e);
                }
            }
            None => {
                if let Some(e) = entry {
                    system_group.entries.push(e);
                }
            }
        }
    }

    // If there was an unclosed group
    if let Some(g) = current_group.take() {
        groups.push(g);
    }

    // Insert system group at the beginning if it has entries
    if !system_group.entries.is_empty() {
        groups.insert(0, system_group);
    }

    Ok(groups)
}

/// Parse a single line into a HostEntry.
///
/// Supports:
/// - `192.168.1.1 example.com` (enabled)
/// - `# 192.168.1.1 example.com` (disabled / commented out)
/// - `# pure comment lines` are skipped (return None)
fn parse_line(line: &str) -> Option<HostEntry> {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return None;
    }

    let (enabled, rest) = if let Some(stripped) = trimmed.strip_prefix('#') {
        (false, stripped.trim())
    } else {
        (true, trimmed)
    };

    if rest.is_empty() {
        return None;
    }

    // Split inline comment first, then parse mapping tokens with collapsed whitespace.
    let (mapping, comment) = if let Some((mapping, comment)) = rest.split_once('#') {
        (mapping.trim(), comment.trim().to_string())
    } else {
        (rest, String::new())
    };

    let mut parts = mapping.split_whitespace();

    let ip = match parts.next() {
        Some(ip) if is_valid_ip_like(ip) => ip.to_string(),
        _ => return None, // Not a host entry line (pure comment)
    };

    let hostname = match parts.next() {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return None,
    };

    Some(HostEntry::new(ip, hostname, comment, enabled))
}

/// Basic check for IP-like string (v4 or v6 prefix).
fn is_valid_ip_like(s: &str) -> bool {
    // IPv4: starts with digit
    // IPv6: starts with hex char or ::
    let first = s.chars().next().unwrap_or(' ');
    first.is_ascii_digit() || first == ':' || s.contains(':')
}

/// Serialize groups to clean hosts file content (no @group/@end markers).
/// Only writes enabled groups and entries.
pub fn serialize_groups_clean(groups: &[HostGroup]) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("# Managed by Guohe Hosts - Do not edit manually".to_string());
    lines.push(String::new());

    for group in groups {
        if !group.enabled {
            continue;
        }

        lines.push(format!("# [{}]", group.name));

        for entry in &group.entries {
            if !entry.enabled {
                continue;
            }
            let comment_part = if entry.comment.is_empty() {
                String::new()
            } else {
                format!(" # {}", entry.comment)
            };
            lines.push(format!("{} {}{}", entry.ip, entry.hostname, comment_part));
        }

        lines.push(String::new());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::parse_hosts_content;

    #[test]
    fn parses_entries_with_multiple_spaces() {
        let content = r#"
10.0.0.1     example.com
  10.0.0.2        another.example   # test comment
# 10.0.0.3      disabled.example
"#;

        let groups = parse_hosts_content(content).expect("should parse content");
        let entries: Vec<_> = groups.into_iter().flat_map(|g| g.entries).collect();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].ip, "10.0.0.1");
        assert_eq!(entries[0].hostname, "example.com");
        assert_eq!(entries[0].comment, "");
        assert!(entries[0].enabled);

        assert_eq!(entries[1].ip, "10.0.0.2");
        assert_eq!(entries[1].hostname, "another.example");
        assert_eq!(entries[1].comment, "test comment");
        assert!(entries[1].enabled);

        assert_eq!(entries[2].ip, "10.0.0.3");
        assert_eq!(entries[2].hostname, "disabled.example");
        assert!(!entries[2].enabled);
    }
}
