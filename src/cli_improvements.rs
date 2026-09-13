use std::io::{self, Write};

pub fn print_banner() {
    println!("project-memory v0.2.0");
    println!("Context-aware memory for AI agents");
    println!();
}

pub fn print_success(message: &str) {
    println!("[ok] {}", message);
}

pub fn print_error(message: &str) {
    eprintln!("[error] {}", message);
}

pub fn print_warning(message: &str) {
    println!("[warn] {}", message);
}

pub fn print_info(message: &str) {
    println!("[info] {}", message);
}

pub fn print_step(step: usize, total: usize, message: &str) {
    println!("[{}/{}] {}", step, total, message);
}

pub fn confirm_action(message: &str) -> bool {
    print!("{} (y/n): ", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y"
}

pub fn select_from_list(items: &[&str], prompt: &str) -> Option<usize> {
    println!("{}", prompt);
    for (i, item) in items.iter().enumerate() {
        println!("  {}. {}", i + 1, item);
    }
    println!("  0. Cancel");
    
    print!("Choice: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    match input.trim().parse::<usize>() {
        Ok(0) => None,
        Ok(n) if n <= items.len() => Some(n - 1),
        _ => None,
    }
}

pub fn print_memory_card(memory: &crate::types::Memory, show_full: bool) {
    let _kind_color = match memory.kind {
        crate::types::MemoryKind::Convention => "cyan",
        crate::types::MemoryKind::Pattern => "green",
        crate::types::MemoryKind::Decision => "yellow",
        crate::types::MemoryKind::Preference => "magenta",
        crate::types::MemoryKind::Context => "blue",
    };
    
    println!("  [{}] {}", memory.kind, memory.key);
    if show_full {
        println!("    {}", memory.content);
        if !memory.tags.is_empty() {
            println!("    Tags: {}", memory.tags.join(", "));
        }
        if !memory.related_ids.is_empty() {
            println!("    Related: {} memories", memory.related_ids.len());
        }
    } else {
        let truncated = if memory.content.len() > 80 {
            format!("{}...", &memory.content[..80])
        } else {
            memory.content.clone()
        };
        println!("    {}", truncated);
    }
}

pub fn print_memory_table(memories: &[crate::types::Memory]) {
    if memories.is_empty() {
        println!("No memories found.");
        return;
    }
    
    println!("{:<12} {:<30} {:<50}", "Kind", "Key", "Content");
    println!("{}", "-".repeat(92));
    
    for m in memories {
        let content = if m.content.len() > 50 {
            format!("{}...", &m.content[..50])
        } else {
            m.content.clone()
        };
        println!("{:<12} {:<30} {:<50}", m.kind, m.key, content);
    }
}

pub fn print_stats_summary(memories: &[crate::types::Memory]) {
    let total = memories.len();
    let with_tags = memories.iter().filter(|m| !m.tags.is_empty()).count();
    let with_links = memories.iter().filter(|m| !m.related_ids.is_empty()).count();
    let pinned = memories.iter().filter(|m| m.tags.contains(&"pinned".to_string())).count();
    
    println!("Summary:");
    println!("  Total: {} memories", total);
    println!("  With tags: {} ({:.0}%)", with_tags, (with_tags as f64 / total.max(1) as f64) * 100.0);
    println!("  With links: {} ({:.0}%)", with_links, (with_links as f64 / total.max(1) as f64) * 100.0);
    println!("  Pinned: {}", pinned);
}
