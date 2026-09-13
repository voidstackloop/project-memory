use std::path::Path;

pub fn show_quick_start(_project_dir: &Path) {
    println!("Quick Start Guide");
    println!("=================\n");
    
    println!("1. Initialize project memory:");
    println!("   pmem init\n");
    
    println!("2. Apply a template (optional):");
    println!("   pmem template list");
    println!("   pmem template apply rust-lib\n");
    
    println!("3. Add memories:");
    println!("   pmem add --kind convention -k \"naming\" -c \"Use snake_case\" --tags rust");
    println!("   pmem add --kind decision -k \"db\" -c \"Using SQLite\" --tags db\n");
    
    println!("4. Search memories:");
    println!("   pmem search \"naming\"");
    println!("   pmem fts-search \"snake_case\"\n");
    
    println!("5. Browse memories:");
    println!("   pmem browse");
    println!("   pmem list\n");
    
    println!("6. Start MCP server for AI agents:");
    println!("   pmem stdio\n");
    
    println!("7. Generate documentation:");
    println!("   pmem docs --format markdown\n");
    
    println!("8. Create backups:");
    println!("   pmem backup\n");
    
    println!("For more help on any command:");
    println!("   pmem <command> --help");
}

pub fn show_tips() {
    println!("Tips & Tricks");
    println!("=============\n");
    
    println!("1. Use ID prefixes:");
    println!("   Instead of full UUID, use first 8 chars:");
    println!("   pmem get f3adf2aa\n");
    
    println!("2. Use fuzzy search:");
    println!("   pmem search \"snke\" will find \"snake_case\"");
    println!("   pmem search \"db\" will find \"database\"\n");
    
    println!("3. Pin important memories:");
    println!("   pmem pin <id>");
    println!("   pmem pinned\n");
    
    println!("4. Link related memories:");
    println!("   pmem link <id1> <id2>");
    println!("   pmem related <id>\n");
    
    println!("5. Use groups:");
    println!("   pmem group create rust-conventions");
    println!("   pmem group add rust-conventions <id>\n");
    
    println!("6. Backup before major changes:");
    println!("   pmem backup");
    println!("   pmem snapshot save -n \"before-refactor\"\n");
    
    println!("7. Export for sharing:");
    println!("   pmem export-format --format yaml");
    println!("   pmem docs --format markdown\n");
    
    println!("8. Use FTS for complex queries:");
    println!("   pmem fts-search \"error AND handling\"");
    println!("   pmem fts-search \"rust OR python\"\n");
}
