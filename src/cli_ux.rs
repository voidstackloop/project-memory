use std::io::{self, Write};

pub struct ProgressBar {
    total: usize,
    current: usize,
    width: usize,
    label: String,
}

impl ProgressBar {
    pub fn new(total: usize, label: &str) -> Self {
        Self {
            total,
            current: 0,
            width: 40,
            label: label.to_string(),
        }
    }

    pub fn set(&mut self, current: usize) {
        self.current = current;
        self.draw();
    }

    pub fn increment(&mut self) {
        self.current += 1;
        self.draw();
    }

    pub fn finish(&self) {
        print!("\r{} [{}] {}/{} (100%)   \n",
            self.label,
            "=".repeat(self.width),
            self.total,
            self.total
        );
        io::stdout().flush().unwrap();
    }

    fn draw(&self) {
        let percent = self.current as f64 / self.total as f64;
        let filled = (self.width as f64 * percent) as usize;
        let empty = self.width - filled;

        print!("\r{} [{}>{}] {}/{} ({:.0}%)   ",
            self.label,
            "=".repeat(filled),
            " ".repeat(empty),
            self.current,
            self.total,
            percent * 100.0
        );
        io::stdout().flush().unwrap();
    }
}

pub struct Spinner {
    frames: Vec<&'static str>,
    current: usize,
    message: String,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current: 0,
            message: message.to_string(),
        }
    }

    pub fn tick(&mut self) {
        print!("\r{} {}", self.frames[self.current], self.message);
        io::stdout().flush().unwrap();
        self.current = (self.current + 1) % self.frames.len();
    }

    pub fn success(&self, message: &str) {
        println!("\r[ok] {}", message);
    }

    pub fn error(&self, message: &str) {
        println!("\r[error] {}", message);
    }
}

pub fn prompt_select(prompt: &str, options: &[&str]) -> Option<usize> {
    println!("{}", prompt);
    for (i, option) in options.iter().enumerate() {
        println!("  {}. {}", i + 1, option);
    }
    println!("  0. Cancel");

    print!("\nChoice: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim().parse::<usize>() {
        Ok(0) => None,
        Ok(n) if n <= options.len() => Some(n - 1),
        _ => None,
    }
}

pub fn prompt_confirm(prompt: &str, default: bool) -> bool {
    let hint = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", prompt, hint);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();

    if input.is_empty() {
        default
    } else {
        input == "y" || input == "yes"
    }
}

pub fn prompt_input(prompt: &str, default: &str) -> String {
    if default.is_empty() {
        print!("{}: ", prompt);
    } else {
        print!("{} [{}]: ", prompt, default);
    }
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_string();

    if input.is_empty() {
        default.to_string()
    } else {
        input
    }
}

pub fn print_box(title: &str, content: &str) {
    let width = 60;
    let border = "─".repeat(width - 2);

    println!("┌{}┐", border);
    println!("│ {:<width$} │", title, width = width - 4);
    println!("├{}┤", border);

    for line in content.lines() {
        println!("│ {:<width$} │", line, width = width - 4);
    }

    println!("└{}┘", border);
}

pub fn print_tree(items: &[(&str, Vec<&str>)]) {
    for (i, (parent, children)) in items.iter().enumerate() {
        let is_last = i == items.len() - 1;
        let prefix = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        println!("{}{}", prefix, parent);

        for (j, child) in children.iter().enumerate() {
            let is_last_child = j == children.len() - 1;
            let child_branch = if is_last_child { "└── " } else { "├── " };
            println!("{}{}{}", child_prefix, child_branch, child);
        }
    }
}

pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::from("No data.");
    }

    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let mut output = String::new();

    // Header
    for (i, header) in headers.iter().enumerate() {
        output.push_str(&format!("{:width$}  ", header, width = widths[i]));
    }
    output.push('\n');

    // Separator
    for width in &widths {
        output.push_str(&format!("{}  ", "─".repeat(*width)));
    }
    output.push('\n');

    // Rows
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                output.push_str(&format!("{:width$}  ", cell, width = widths[i]));
            }
        }
        output.push('\n');
    }

    output
}
