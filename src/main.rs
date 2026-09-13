use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Shell};
use colored::Colorize;
use project_memory::audit;
use project_memory::config::Config;
use project_memory::git::resolve_project_dir;
use project_memory::store::MemoryStore;
use project_memory::templates;
use project_memory::types::{MemoryInput, MemoryKind};
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pmem", version, about = "Context-aware project memory for AI agents")]
struct Cli {
    /// Path to project root (defaults to auto-detection via git)
    #[arg(long, global = true)]
    project: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize .memory/ in the current project
    Init,

    /// Add a new memory entry
    Add {
        /// Type: convention, pattern, decision, preference, context
        #[arg(long)]
        kind: MemoryKind,

        /// Short key/name for the memory
        #[arg(short, long)]
        key: String,

        /// The memory content
        #[arg(short, long)]
        content: String,

        /// Comma-separated tags
        #[arg(short, long, default_value = "")]
        tags: String,

        /// Comma-separated related memory IDs
        #[arg(long, default_value = "")]
        related: String,
    },

    /// List memories
    List {
        /// Filter by kind
        #[arg(short, long)]
        kind: Option<MemoryKind>,

        /// Max results
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Search memories with fuzzy matching
    Search {
        /// Search query
        query: String,

        /// Max results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Use exact matching instead of fuzzy
        #[arg(long)]
        exact: bool,
    },

    /// Show a single memory by ID
    Get {
        /// Memory ID (or prefix)
        id: String,
    },

    /// Update an existing memory
    Update {
        /// Memory ID
        id: String,

        /// Type: convention, pattern, decision, preference, context
        #[arg(long)]
        kind: MemoryKind,

        /// Short key/name
        #[arg(short, long)]
        key: String,

        /// The memory content
        #[arg(short, long)]
        content: String,

        /// Comma-separated tags
        #[arg(short, long, default_value = "")]
        tags: String,
    },

    /// Delete a memory by ID
    Delete {
        /// Memory ID
        id: String,
    },

    /// Link two related memories
    Link {
        /// Source memory ID
        source: String,

        /// Target memory ID
        target: String,
    },

    /// Unlink two memories
    Unlink {
        /// Source memory ID
        source: String,

        /// Target memory ID
        target: String,
    },

    /// Show related memories
    Related {
        /// Memory ID
        id: String,
    },

    /// Export all memories as JSON
    Export,

    /// Import memories from JSON file
    Import {
        /// Path to JSON file
        file: PathBuf,
    },

    /// Show memory stats
    Stats,

    /// Browse memories interactively (TUI)
    Browse,

    /// Start MCP server (HTTP) for AI agent integration
    Serve {
        /// Port to listen on
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Start MCP server on stdio (for Claude Code, Cursor, etc.)
    Stdio,

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Apply a memory template to your project
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// Manage tags
    Tags {
        #[command(subcommand)]
        action: TagsAction,
    },

    /// Show audit history
    History {
        /// Memory ID to filter by
        #[arg(short, long)]
        id: Option<String>,

        /// Max results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Batch operations on multiple memories
    Batch {
        #[command(subcommand)]
        action: BatchAction,
    },

    /// Manage snapshots (save, list, diff)
    Snapshot {
        #[command(subcommand)]
        action: SnapshotAction,
    },

    /// Find and merge duplicate memories
    Dedupe {
        #[command(subcommand)]
        action: Option<DedupeAction>,

        /// Similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.8")]
        threshold: f64,
    },

    /// Start web dashboard
    Dashboard {
        /// Port to listen on
        #[arg(short, long, default_value = "3778")]
        port: u16,
    },

    /// Add a memory interactively (prompt-based)
    AddInteractive,

    /// Merge memories from another project
    Merge {
        /// Path to other project
        other: PathBuf,

        /// Strategy: skip, overwrite, keep-both
        #[arg(long, default_value = "skip")]
        strategy: String,
    },

    /// Validate memory quality
    Validate {
        /// Auto-fix issues
        #[arg(long)]
        fix: bool,
    },

    /// Manage memory groups
    Group {
        #[command(subcommand)]
        action: GroupAction,
    },

    /// Export memory relationship graph
    Graph {
        /// Output format: dot or json
        #[arg(long, default_value = "dot")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import memories from markdown file
    ImportMd {
        /// Path to markdown file
        file: PathBuf,
    },

    /// Start REST API server
    Api {
        /// Port to listen on
        #[arg(short, long, default_value = "3779")]
        port: u16,
    },

    /// Pin important memories
    Pin {
        /// Memory ID
        id: String,
    },

    /// Unpin a memory
    Unpin {
        /// Memory ID
        id: String,
    },

    /// List pinned memories
    Pinned,

    /// Search memories by date range
    DateSearch {
        /// After date (RFC3339 or YYYY-MM-DD)
        #[arg(long)]
        after: Option<String>,

        /// Before date (RFC3339 or YYYY-MM-DD)
        #[arg(long)]
        before: Option<String>,

        /// Max results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Manage git hooks
    Hooks {
        #[command(subcommand)]
        action: Option<HooksAction>,
    },

    /// Import memories from code comments (TODO/FIXME)
    ImportCode {
        /// Directory to scan
        #[arg(short, long)]
        dir: Option<String>,

        /// File extensions to scan (comma-separated)
        #[arg(long)]
        extensions: Option<String>,
    },

    /// Manage encryption for sensitive memories
    Encryption {
        #[command(subcommand)]
        action: EncryptionAction,
    },





    /// Show memory analytics
    Analytics {
        /// Output format: text, json
        #[arg(long)]
        format: Option<String>,
    },

    /// Generate documentation from memories
    Docs {
        /// Output format: markdown, html, json
        #[arg(long)]
        format: Option<String>,
    },

    /// Faceted search with multiple filters
    FacetedSearch {
        /// Search query
        #[arg(long)]
        query: Option<String>,

        /// Filter by kind
        #[arg(long)]
        kind: Option<String>,

        /// Filter by tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Filter by pinned
        #[arg(long)]
        pinned: Option<bool>,

        /// Max results
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },

    /// Manage webhooks
    Webhook {
        #[command(subcommand)]
        action: WebhookAction,
    },

    /// Sync memories with remote project
    Sync {
        /// Remote project path
        remote: String,
        /// Sync strategy: local, remote, newer, both
        #[arg(long, default_value = "both")]
        strategy: String,
    },

    /// Export sync bundle
    SyncExport,

    /// Import sync bundle
    SyncImport {
        /// Bundle file path
        file: String,
        /// Import strategy: skip, overwrite
        #[arg(long, default_value = "skip")]
        strategy: String,
    },

    /// Show advanced analytics
    AnalyticsV2,

    /// Manage custom templates
    CustomTemplate {
        #[command(subcommand)]
        action: CustomTemplateAction,
    },

    /// Watch for changes
    Watch {
        /// Watch interval in seconds
        #[arg(short, long)]
        interval: Option<u64>,
    },

    /// Show watch status
    WatchStatus,

    /// Advanced search with filters
    AdvancedSearch {
        /// Search query
        query: String,
        /// Filter by kind
        #[arg(long)]
        kind: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Filter pinned only
        #[arg(long)]
        pinned: bool,
        /// Max results
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Get search suggestions
    SearchSuggest {
        /// Partial query
        query: String,
    },

    /// Export as HTML report
    ExportHtml {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Export JSON schema
    ExportSchema,

    /// Validate memories v2
    ValidateV2 {
        /// Auto-fix issues
        #[arg(long)]
        fix: bool,
    },

    /// Export memories to various formats
    ExportFormat {
        /// Output format: csv, yaml, toml, json
        #[arg(long)]
        format: Option<String>,
    },

    /// Search with highlighting
    HighlightSearch {
        /// Search query
        query: String,
    },

    /// Create a backup
    Backup,

    /// List backups
    BackupList,

    /// Restore a backup
    BackupRestore {
        /// Backup name
        name: String,
    },

    /// Clean up old backups
    BackupCleanup {
        /// Number of backups to keep
        #[arg(long, default_value = "5")]
        keep: usize,
    },

    /// Migrate memories from other tools
    Migrate {
        /// Source path (file or directory)
        source: String,

        /// Source tool: obsidian, notion, json, markdown, auto
        #[arg(long)]
        tool: Option<String>,
    },

    /// Show memory graph statistics
    GraphStats,

    /// Find shortest path between two memories
    GraphPath {
        /// Source memory ID
        from: String,
        /// Target memory ID
        to: String,
    },

    /// Show connected components in memory graph
    GraphComponents,

    /// Full-text search using SQLite FTS5
    FtsSearch {
        /// Search query (FTS5 syntax)
        query: String,
        /// Max results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Rebuild FTS index
    FtsRebuild,

    /// Get search suggestions
    Suggestions {
        /// Partial query
        query: String,
    },

    /// Show popular search terms
    PopularSearches,

    /// Show quick start guide
    QuickStart,

    /// Show tips and tricks
    Tips,

    /// Validate memories with improved checks
    ValidateImproved {
        /// Auto-fix issues
        #[arg(long)]
        fix: bool,
    },

    /// Show improved statistics
    StatsImproved,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current config
    Show,
    /// Reset config to defaults
    Reset,
    /// Set a config value
    Set {
        /// Config key
        key: String,
        /// Config value
        value: String,
    },
}

#[derive(Subcommand)]
enum PluginAction {
    /// List installed plugins
    List,
    /// Create example plugin
    Create,
    /// Enable a plugin
    Enable { name: String },
    /// Disable a plugin
    Disable { name: String },
    /// Remove a plugin
    Remove { name: String },
}

#[derive(Subcommand)]
enum WebhookAction {
    /// List configured webhooks
    List,
    /// Add a webhook
    Add {
        /// Webhook name
        name: String,
        /// Webhook URL
        url: String,
        /// Comma-separated events (memory_added, memory_deleted, memory_updated)
        events: String,
    },
    /// Remove a webhook
    Remove { name: String },
}

#[derive(Subcommand)]
enum CustomTemplateAction {
    /// List custom templates
    List,
    /// Create template from current memories
    Create {
        /// Template name
        name: String,
        /// Template description
        description: String,
    },
    /// Apply a custom template
    Apply { name: String },
    /// Delete a custom template
    Delete { name: String },
}


#[derive(Subcommand)]
enum TemplateAction {
    /// List available templates
    List,
    /// Show template details
    Show {
        /// Template name
        name: String,
    },
    /// Apply a template to your project
    Apply {
        /// Template name
        name: String,
    },
}

#[derive(Subcommand)]
enum TagsAction {
    /// List all tags with counts
    List,
    /// Rename a tag across all memories
    Rename {
        /// Old tag name
        old: String,
        /// New tag name
        new: String,
    },
    /// Delete a tag from all memories
    Delete {
        /// Tag name
        tag: String,
    },
    /// Add a tag to a memory
    Add {
        /// Memory ID
        id: String,
        /// Tag name
        tag: String,
    },
    /// Show detailed tag statistics
    Stats,
}

#[derive(Subcommand)]
enum BatchAction {
    /// Delete multiple memories by IDs
    Delete {
        /// Comma-separated memory IDs
        ids: String,
    },
    /// Add a tag to multiple memories
    Tag {
        /// Comma-separated memory IDs
        ids: String,
        /// Tag to add
        tag: String,
    },
    /// Change kind for multiple memories
    Kind {
        /// Comma-separated memory IDs
        ids: String,
        /// New kind
        kind: MemoryKind,
    },
}

#[derive(Subcommand)]
enum SnapshotAction {
    /// Save a new snapshot
    Save {
        /// Snapshot name
        #[arg(short, long)]
        name: String,
    },
    /// List all snapshots
    List,
    /// Show snapshot details
    Show {
        /// Snapshot ID or name
        id: String,
    },
    /// Compare current state with a snapshot
    Diff {
        /// Snapshot ID or name to compare with
        id: String,
    },
    /// Restore memories from a snapshot
    Restore {
        /// Snapshot ID or name
        id: String,
    },
}

#[derive(Subcommand)]
enum DedupeAction {
    /// Find duplicates
    Find,
    /// Merge all duplicate groups
    Merge,
    /// Merge a specific group by index
    MergeGroup {
        /// Group index
        index: usize,
    },
}

#[derive(Subcommand)]
enum GroupAction {
    /// Create a new group
    Create {
        /// Group name
        name: String,
    },
    /// List all groups
    List,
    /// Add a memory to a group
    Add {
        /// Group name
        group: String,
        /// Memory ID
        id: String,
    },
    /// Remove a memory from a group
    Remove {
        /// Group name
        group: String,
        /// Memory ID
        id: String,
    },
    /// List memories in a group
    Show {
        /// Group name
        name: String,
    },
}

#[derive(Subcommand)]
enum HooksAction {
    /// Install git hooks
    Install,
    /// Uninstall git hooks
    Uninstall,
    /// Show hooks status
    Status,
}

#[derive(Subcommand)]
enum EncryptionAction {
    /// Set encryption key
    SetKey {
        /// Encryption key
        key: String,
    },
    /// Remove encryption key
    RemoveKey,
    /// Show encryption status
    Status,
    /// Encrypt a memory
    Encrypt {
        /// Memory ID
        id: String,
    },
    /// Decrypt a memory
    Decrypt {
        /// Memory ID
        id: String,
    },
}


fn open_store(project_dir: &std::path::Path) -> MemoryStore {
    MemoryStore::open_in_project(project_dir).expect("Failed to open memory store. Run `pmem init` first.")
}

fn resolve_id(store: &MemoryStore, id: &str) -> String {
    if store.get(id).ok().flatten().is_some() {
        return id.to_string();
    }
    let all = store.list(None, 10000).unwrap_or_default();
    let matches: Vec<_> = all.iter().filter(|m| m.id.starts_with(id)).collect();
    if matches.len() == 1 {
        return matches[0].id.clone();
    }
    id.to_string()
}

fn parse_ids(ids_str: &str) -> Vec<String> {
    ids_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let project_dir = resolve_project_dir(&cli.project);

    match cli.command {
        Command::Init => {
            let memory_dir = project_dir.join(".memory");
            if memory_dir.exists() {
                println!("{} .memory/ already exists in {}", "[ok]".green(), project_dir.display());
            } else {
                std::fs::create_dir_all(&memory_dir).expect("Failed to create .memory/");
                let _ = MemoryStore::open_in_project(&project_dir);
                Config::default().save(&project_dir).expect("Failed to write config");
                println!("{} Initialized .memory/ in {}", "[ok]".green(), project_dir.display());
                println!();
                println!("  {}", "Next steps:".bold());
                println!("  pmem template list         # Browse templates");
                println!("  pmem template apply rust   # Apply a template");
                println!("  pmem add --kind convention -k \"naming\" -c \"Use snake_case\" --tags rust");
                println!("  pmem browse                # Interactive TUI");
                println!("  pmem stdio                 # MCP server for AI agents");
            }
        }

        Command::Add { kind, key, content, tags, related } => {
            let store = open_store(&project_dir);
            let config = Config::load(&project_dir);

            let mut all_tags: Vec<String> = tags
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
            for dt in &config.default_tags {
                if !all_tags.contains(dt) {
                    all_tags.push(dt.clone());
                }
            }

            let related_ids = parse_ids(&related);

            let memory = store
                .add(MemoryInput { kind, key, content, tags: all_tags, related_ids })
                .expect("Failed to add memory");

            let _ = audit::log(store.conn(), "add", &memory.id, &format!("{}: {}", memory.kind, memory.key));

            println!("{} Added memory {}", "[ok]".green(), memory.id[..8].to_string().dimmed());
            println!("  {} {}", "kind:".dimmed(), memory.kind);
            println!("  {} {}", "key:".dimmed(), memory.key.bold());
            println!("  {} {}", "content:".dimmed(), memory.content);
            if !memory.tags.is_empty() {
                println!("  {} {}", "tags:".dimmed(), memory.tags.join(", "));
            }
        }

        Command::List { kind, limit } => {
            let store = open_store(&project_dir);
            let config = Config::load(&project_dir);
            let limit = limit.unwrap_or(config.max_results);
            let memories = store.list(kind, limit).expect("Failed to list memories");

            if memories.is_empty() {
                println!("{} No memories found. Add some with `pmem add`", "[i]".blue());
                return;
            }

            println!("{} {} memories found\n", "[i]".blue(), memories.len());
            for m in &memories {
                let tags_str = if m.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", m.tags.join(", "))
                };
                let related_str = if m.related_ids.is_empty() {
                    String::new()
                } else {
                    format!(" →{}", m.related_ids.len())
                };
                println!(
                    "  {} {}{}{} {}",
                    format!("[{}]", m.kind).cyan(),
                    m.key.bold(),
                    tags_str.dimmed(),
                    related_str.dimmed(),
                    format!("({})", &m.id[..8]).dimmed()
                );
                println!("    {}", m.content);
            }
        }

        Command::Search { query, limit, exact } => {
            let store = open_store(&project_dir);

            if exact {
                let memories = store.search(&query, limit).expect("Failed to search memories");
                if memories.is_empty() {
                    println!("{} No memories matching '{}'", "[i]".blue(), query);
                    return;
                }
                println!("{} {} exact results for '{}'\n", "[i]".blue(), memories.len(), query);
                for m in &memories {
                    println!("  {} {} {}", format!("[{}]", m.kind).cyan(), m.key.bold(), format!("({})", &m.id[..8]).dimmed());
                    println!("    {}", m.content);
                }
            } else {
                let scored = store.fuzzy_search(&query, limit).expect("Failed to search memories");
                if scored.is_empty() {
                    println!("{} No memories matching '{}'", "[i]".blue(), query);
                    return;
                }
                println!("{} {} results for '{}'\n", "[i]".blue(), scored.len(), query);
                for sm in &scored {
                    let m = &sm.memory;
                    println!(
                        "  {} {} {} {}",
                        format!("[{}]", m.kind).cyan(),
                        m.key.bold(),
                        format!("({:.2})", sm.score).yellow(),
                        format!("({})", &m.id[..8]).dimmed()
                    );
                    println!("    {}", m.content);
                }
            }
        }

        Command::Get { id } => {
            let store = open_store(&project_dir);
            let id = resolve_id(&store, &id);
            match store.get(&id).expect("Failed to get memory") {
                Some(m) => {
                    println!("{}: {}", "ID".dimmed(), m.id);
                    println!("{}: {}", "Kind".dimmed(), m.kind);
                    println!("{}: {}", "Key".dimmed(), m.key.bold());
                    println!("{}: {}", "Content".dimmed(), m.content);
                    if !m.tags.is_empty() {
                        println!("{}: {}", "Tags".dimmed(), m.tags.join(", "));
                    }
                    if !m.related_ids.is_empty() {
                        println!("{}:", "Related".dimmed());
                        for rid in &m.related_ids {
                            if let Ok(Some(r)) = store.get(rid) {
                                println!("  - {} {}", format!("[{}]", r.kind).cyan(), r.key);
                            }
                        }
                    }
                    println!("{}: {}", "Created".dimmed(), m.created_at);
                    println!("{}: {}", "Updated".dimmed(), m.updated_at);
                }
                None => {
                    println!("{} Memory '{}' not found", "[x]".red(), id);
                }
            }
        }

        Command::Update { id, kind, key, content, tags } => {
            let store = open_store(&project_dir);
            let id = resolve_id(&store, &id);
            let existing = store.get(&id).expect("Failed to get memory");
            let related_ids = existing.as_ref().map(|m| m.related_ids.clone()).unwrap_or_default();

            let tags: Vec<String> = tags
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            match store
                .update(&id, MemoryInput { kind, key, content, tags, related_ids })
                .expect("Failed to update memory")
            {
                Some(m) => {
                    let _ = audit::log(store.conn(), "update", &m.id, &m.key);
                    println!("{} Updated memory {}", "[ok]".green(), m.id[..8].to_string().dimmed());
                }
                None => {
                    println!("{} Memory '{}' not found", "[x]".red(), id);
                }
            }
        }

        Command::Delete { id } => {
            let store = open_store(&project_dir);
            let id = resolve_id(&store, &id);
            if store.delete(&id).expect("Failed to delete memory") {
                let _ = audit::log(store.conn(), "delete", &id, "");
                println!("{} Deleted memory {}", "[ok]".green(), id[..8].to_string().dimmed());
            } else {
                println!("{} Memory '{}' not found", "[x]".red(), id);
            }
        }

        Command::Link { source, target } => {
            let store = open_store(&project_dir);
            let source = resolve_id(&store, &source);
            let target = resolve_id(&store, &target);
            if store.link(&source, &target).expect("Failed to link memories") {
                let _ = audit::log(store.conn(), "link", &source, &target);
                println!("{} Linked {} → {}", "[ok]".green(), &source[..8], &target[..8]);
            } else {
                println!("{} One or both memory IDs not found", "[x]".red());
            }
        }

        Command::Unlink { source, target } => {
            let store = open_store(&project_dir);
            let source = resolve_id(&store, &source);
            let target = resolve_id(&store, &target);
            store.unlink(&source, &target).expect("Failed to unlink memories");
            let _ = audit::log(store.conn(), "unlink", &source, &target);
            println!("{} Unlinked {} → {}", "[ok]".green(), &source[..8], &target[..8]);
        }

        Command::Related { id } => {
            let store = open_store(&project_dir);
            let id = resolve_id(&store, &id);
            let related = store.get_related(&id).expect("Failed to get related memories");
            if related.is_empty() {
                println!("{} No related memories for '{}'", "[i]".blue(), &id[..8]);
            } else {
                println!("{} {} related memories:\n", "[i]".blue(), related.len());
                for m in &related {
                    println!("  {} {} {}", format!("[{}]", m.kind).cyan(), m.key.bold(), format!("({})", &m.id[..8]).dimmed());
                    println!("    {}", m.content);
                }
            }
        }

        Command::Export => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            println!("{}", serde_json::to_string_pretty(&memories).unwrap());
        }

        Command::Import { file } => {
            let store = open_store(&project_dir);
            let content = std::fs::read_to_string(&file).expect("Failed to read file");
            let inputs: Vec<MemoryInput> = serde_json::from_str(&content).expect("Failed to parse JSON");
            let count = store.import_memories(inputs).expect("Failed to import memories");
            println!("{} Imported {} memories", "[ok]".green(), count);
        }

        Command::Stats => {
            let store = open_store(&project_dir);
            let count = store.count().expect("Failed to get count");
            let by_kind = store.count_by_kind().expect("Failed to get kind counts");
            let tags = store.list_tags().unwrap_or_default();
            let db_path = project_dir.join(".memory").join("store.db");
            let size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
            let history_count = audit::count(store.conn()).unwrap_or(0);

            println!("{} Project Memory Stats", "📊".to_string().bold());
            println!("  {} {}", "Total memories:".dimmed(), count);
            for (kind, cnt) in &by_kind {
                println!("    {} {}", format!("[{kind}]").cyan(), cnt);
            }
            if !tags.is_empty() {
                println!("  {} {} unique tags", "Tags:".dimmed(), tags.len());
            }
            println!("  {} {}", "Audit entries:".dimmed(), history_count);
            println!("  {} {:.2} KB", "DB size:".dimmed(), size as f64 / 1024.0);
            println!("  {} {}", "Location:".dimmed(), db_path.display());
        }

        Command::Browse => {
            project_memory::tui::run_tui(project_dir).expect("TUI failed");
        }

        Command::Serve { port } => {
            let config = Config::load(&project_dir);
            let port = port.unwrap_or(config.mcp_port);
            println!("{} Starting MCP HTTP server on port {}...", "🚀".to_string().green(), port);
            project_memory::mcp::serve(project_dir, port).await;
        }

        Command::Stdio => {
            project_memory::mcp_stdio::serve_stdio(project_dir);
        }

        Command::Config { action } => {
            match action.unwrap_or(ConfigAction::Show) {
                ConfigAction::Show => {
                    let config = Config::load(&project_dir);
                    let path = Config::config_path(&project_dir);
                    println!("{} {}", "Config:".bold(), path.display());
                    println!("{}", toml::to_string_pretty(&config).unwrap());
                }
                ConfigAction::Reset => {
                    Config::default().save(&project_dir).expect("Failed to reset config");
                    println!("{} Config reset to defaults", "[ok]".green());
                }
                ConfigAction::Set { key, value } => {
                    let mut config = Config::load(&project_dir);
                    match key.as_str() {
                        "max_results" => {
                            config.max_results = value.parse().expect("Expected a number");
                        }
                        "mcp_port" => {
                            config.mcp_port = value.parse().expect("Expected a port number");
                        }
                        "default_tags" => {
                            config.default_tags = value.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        "auto_link" => {
                            config.auto_link = value.parse().expect("Expected true/false");
                        }
                        "fuzzy_search" => {
                            config.fuzzy_search = value.parse().expect("Expected true/false");
                        }
                        _ => {
                            println!("{} Unknown config key: '{}'", "[x]".red(), key);
                            println!("  Valid keys: max_results, mcp_port, default_tags, auto_link, fuzzy_search");
                            return;
                        }
                    }
                    config.save(&project_dir).expect("Failed to save config");
                    println!("{} Set {} = {}", "[ok]".green(), key.bold(), value);
                }
            }
        }

        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "pmem", &mut io::stdout());
        }

        // --- Templates ---
        Command::Template { action } => match action {
            TemplateAction::List => {
                let names = templates::list_template_names();
                println!("{} Available templates:\n", "📋".to_string().bold());
                for (name, desc) in &names {
                    println!("  {} {}", format!("{name:<16}").green(), desc.dimmed());
                }
                println!();
                println!("  {} pmem template apply <name>", "Usage:".dimmed());
            }
            TemplateAction::Show { name } => {
                match templates::find_template(&name) {
                    Some(t) => {
                        println!("{} {}", "Template:".bold(), t.name);
                        println!("{} {}", "Description:".dimmed(), t.description);
                        println!("{} {} memories\n", "Contains:".dimmed(), t.memories.len());
                        for m in &t.memories {
                            println!("  {} {} {}", format!("[{}]", m.kind).cyan(), m.key.bold(), m.tags.join(", ").dimmed());
                            println!("    {}", m.content);
                        }
                    }
                    None => {
                        println!("{} Template '{}' not found", "[x]".red(), name);
                        println!("  Run `pmem template list` to see available templates");
                    }
                }
            }
            TemplateAction::Apply { name } => {
                let store = open_store(&project_dir);
                match templates::find_template(&name) {
                    Some(t) => {
                        let inputs: Vec<MemoryInput> = t.memories.iter().map(|m| m.to_input()).collect();
                        let count = store.import_memories(inputs).expect("Failed to apply template");
                        println!("{} Applied template '{}' — {} memories added", "[ok]".green(), name, count);
                    }
                    None => {
                        println!("{} Template '{}' not found", "[x]".red(), name);
                    }
                }
            }
        },

        // --- Tags ---
        Command::Tags { action } => match action {
            TagsAction::List => {
                let store = open_store(&project_dir);
                let tags = store.list_tags().expect("Failed to list tags");
                if tags.is_empty() {
                    println!("{} No tags found", "[i]".blue());
                    return;
                }
                println!("{} {} tags:\n", "🏷️".to_string().bold(), tags.len());
                for ti in &tags {
                    println!("  {} {}", format!("{:<24}", ti.tag).cyan(), format!("({})", ti.count).dimmed());
                }
            }
            TagsAction::Rename { old, new } => {
                let store = open_store(&project_dir);
                let count = store.rename_tag(&old, &new).expect("Failed to rename tag");
                println!("{} Renamed '{}' → '{}' in {} memories", "[ok]".green(), old, new, count);
            }
            TagsAction::Delete { tag } => {
                let store = open_store(&project_dir);
                let count = store.delete_tag(&tag).expect("Failed to delete tag");
                println!("{} Removed tag '{}' from {} memories", "[ok]".green(), tag, count);
            }
            TagsAction::Add { id, tag } => {
                let store = open_store(&project_dir);
                let id = resolve_id(&store, &id);
                if store.add_tag(&id, &tag).expect("Failed to add tag") {
                    println!("{} Added tag '{}' to {}", "[ok]".green(), tag, &id[..8]);
                } else {
                    println!("{} Memory '{}' not found", "[x]".red(), id);
                }
            }
            TagsAction::Stats => {
                let store = open_store(&project_dir);
                let stats = store.tag_stats().expect("Failed to get tag stats");
                if stats.is_empty() {
                    println!("{} No tags found", "[i]".blue());
                    return;
                }
                println!("{} Tag Statistics:\n", "🏷️".to_string().bold());
                for (tag, count, keys) in &stats {
                    println!("  {} {}", format!("{tag:<24}").cyan(), format!("({count} memories)").dimmed());
                    let preview: Vec<_> = keys.iter().take(3).cloned().collect();
                    println!("    {}", preview.join(", ").dimmed());
                }
            }
        },

        // --- History ---
        Command::History { id, limit } => {
            let store = open_store(&project_dir);
            let entries = if let Some(ref memory_id) = id {
                let id = resolve_id(&store, memory_id);
                audit::list_for_memory(store.conn(), &id, limit).unwrap_or_default()
            } else {
                audit::list(store.conn(), limit).unwrap_or_default()
            };

            if entries.is_empty() {
                println!("{} No history entries found", "[i]".blue());
                return;
            }

            println!("{} {} history entries:\n", "📜".to_string().bold(), entries.len());
            for e in &entries {
                let action_str = match e.action.as_str() {
                    "add" => e.action.green(),
                    "update" => e.action.yellow(),
                    "delete" => e.action.red(),
                    "link" => e.action.blue(),
                    "unlink" => e.action.cyan(),
                    _ => e.action.white(),
                };
                println!(
                    "  {} {} {} {}",
                    e.created_at.format("%Y-%m-%d %H:%M").to_string().dimmed(),
                    action_str,
                    &e.memory_id[..8],
                    e.details.dimmed()
                );
            }
        }

        // --- Batch ---
        Command::Batch { action } => match action {
            BatchAction::Delete { ids } => {
                let store = open_store(&project_dir);
                let ids: Vec<String> = ids.split(',').map(|s| {
                    resolve_id(&store, s.trim())
                }).collect();
                let count = store.batch_delete(&ids).expect("Failed to batch delete");
                for id in &ids {
                    let _ = audit::log(store.conn(), "delete", id, "batch");
                }
                println!("{} Deleted {} memories", "[ok]".green(), count);
            }
            BatchAction::Tag { ids, tag } => {
                let store = open_store(&project_dir);
                let ids: Vec<String> = ids.split(',').map(|s| {
                    resolve_id(&store, s.trim())
                }).collect();
                let count = store.batch_add_tag(&ids, &tag).expect("Failed to batch tag");
                println!("{} Added tag '{}' to {} memories", "[ok]".green(), tag, count);
            }
            BatchAction::Kind { ids, kind } => {
                let store = open_store(&project_dir);
                let ids: Vec<String> = ids.split(',').map(|s| {
                    resolve_id(&store, s.trim())
                }).collect();
                let count = store.batch_update_kind(&ids, &kind).expect("Failed to batch update");
                println!("{} Updated {} memories to kind '{}'", "[ok]".green(), count, kind);
            }
        },

        // --- Snapshots ---
        Command::Snapshot { action } => match action {
            SnapshotAction::Save { name } => {
                let store = open_store(&project_dir);
                let memories = store.export_json().expect("Failed to export memories");
                let snapshot = project_memory::snapshot::save_snapshot(&project_dir, &name, &memories)
                    .expect("Failed to save snapshot");
                println!("{} Saved snapshot '{}' ({} memories)", "[ok]".green(), snapshot.name, snapshot.memory_count);
                println!("  {} {}", "ID:".dimmed(), snapshot.id);
            }
            SnapshotAction::List => {
                let snapshots = project_memory::snapshot::list_snapshots(&project_dir)
                    .expect("Failed to list snapshots");
                if snapshots.is_empty() {
                    println!("{} No snapshots found. Create one with `pmem snapshot save -n <name>`", "[i]".blue());
                    return;
                }
                println!("{} {} snapshots:\n", "📸".to_string().bold(), snapshots.len());
                for s in &snapshots {
                    println!("  {} {} {} {}",
                        s.id.green(),
                        s.name.bold(),
                        format!("({} memories)", s.memory_count).dimmed(),
                        s.created_at.format("%Y-%m-%d %H:%M").to_string().dimmed()
                    );
                }
            }
            SnapshotAction::Show { id } => {
                match project_memory::snapshot::load_snapshot(&project_dir, &id) {
                    Ok(Some(s)) => {
                        println!("{}: {}", "Snapshot".bold(), s.name);
                        println!("{}: {}", "ID".dimmed(), s.id);
                        println!("{}: {}", "Created".dimmed(), s.created_at);
                        println!("{}: {}", "Memories".dimmed(), s.memory_count);
                        println!();
                        for m in &s.memories {
                            println!("  {} {} {}", format!("[{}]", m.kind).cyan(), m.key.bold(), format!("({})", &m.id[..8]).dimmed());
                        }
                    }
                    Ok(None) => {
                        println!("{} Snapshot '{}' not found", "[x]".red(), id);
                    }
                    Err(e) => {
                        println!("{} Error loading snapshot: {}", "[x]".red(), e);
                    }
                }
            }
            SnapshotAction::Diff { id } => {
                let store = open_store(&project_dir);
                let current = store.export_json().expect("Failed to export memories");
                match project_memory::snapshot::load_snapshot(&project_dir, &id) {
                    Ok(Some(s)) => {
                        let diff = project_memory::snapshot::diff_snapshots(&s.memories, &current);
                        println!("{} Diff: snapshot '{}' vs current\n", "📊".to_string().bold(), s.name);
                        println!("{}", project_memory::snapshot::format_diff(&diff));
                    }
                    Ok(None) => {
                        println!("{} Snapshot '{}' not found", "[x]".red(), id);
                    }
                    Err(e) => {
                        println!("{} Error: {}", "[x]".red(), e);
                    }
                }
            }
            SnapshotAction::Restore { id } => {
                let store = open_store(&project_dir);
                match project_memory::snapshot::load_snapshot(&project_dir, &id) {
                    Ok(Some(s)) => {
                        // Atomically replace the store with the snapshot, preserving each
                        // memory's original id so related_ids captured in it still resolve.
                        match store.replace_all(&s.memories) {
                            Ok(count) => println!("{} Restored snapshot '{}' — {} memories", "[ok]".green(), s.name, count),
                            Err(e) => println!("{} Failed to restore snapshot: {}", "[x]".red(), e),
                        }
                    }
                    Ok(None) => {
                        println!("{} Snapshot '{}' not found", "[x]".red(), id);
                    }
                    Err(e) => {
                        println!("{} Error: {}", "[x]".red(), e);
                    }
                }
            }
        },

        // --- Dedupe ---
        Command::Dedupe { action, threshold } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let groups = project_memory::duplicates::find_duplicates(&memories, threshold);

            match action.unwrap_or(DedupeAction::Find) {
                DedupeAction::Find => {
                    println!("{}", project_memory::duplicates::format_duplicates(&groups));
                }
                DedupeAction::Merge => {
                    if groups.is_empty() {
                        println!("{} No duplicates found", "[i]".blue());
                        return;
                    }
                    let mut total_merged = 0;
                    let mut errors = 0;
                    for group in &groups {
                        let merged = project_memory::duplicates::merge_group(group);
                        let remove_ids: Vec<String> = group.memories.iter()
                            .filter(|m| m.id != merged.id)
                            .map(|m| m.id.clone())
                            .collect();
                        match store.merge_duplicates(&merged, &remove_ids) {
                            Ok(n) => total_merged += n,
                            Err(e) => { errors += 1; eprintln!("{} Failed to merge group for '{}': {}", "[x]".red(), merged.key, e); }
                        }
                    }
                    if errors > 0 {
                        println!("{} Merged {} of {} duplicate groups ({} memories removed, {} error(s))", "[!]".yellow(), groups.len() - errors, groups.len(), total_merged, errors);
                    } else {
                        println!("{} Merged {} duplicate groups ({} memories removed)", "[ok]".green(), groups.len(), total_merged);
                    }
                }
                DedupeAction::MergeGroup { index } => {
                    if groups.is_empty() {
                        println!("{} No duplicate groups found", "[i]".blue());
                        return;
                    }
                    if index >= groups.len() {
                        println!("{} Invalid group index (max: {})", "[x]".red(), groups.len() - 1);
                        return;
                    }
                    let group = &groups[index];
                    let merged = project_memory::duplicates::merge_group(group);
                    let remove_ids: Vec<String> = group.memories.iter()
                        .filter(|m| m.id != merged.id)
                        .map(|m| m.id.clone())
                        .collect();
                    match store.merge_duplicates(&merged, &remove_ids) {
                        Ok(_) => println!("{} Merged group {} into '{}'", "[ok]".green(), index, merged.key),
                        Err(e) => println!("{} Failed to merge group {}: {}", "[x]".red(), index, e),
                    }
                }
            }
        },

        // --- Dashboard ---
        Command::Dashboard { port } => {
            println!("{} Starting web dashboard on port {}...", "🌐".to_string().green(), port);
            project_memory::dashboard::serve_dashboard(project_dir, port).await;
        }

        // --- Interactive Add ---
        Command::AddInteractive => {
            let store = open_store(&project_dir);
            let config = Config::load(&project_dir);

            println!("{}", "Add a new memory:".bold());
            println!();

            // Kind selection
            println!("  {}", "Select kind:".dimmed());
            for (i, kind) in MemoryKind::all().iter().enumerate() {
                println!("    {} {}", format!("{}.", i + 1).cyan(), kind);
            }
            print!("  {} ", "Choice (1-5):".bold());
            io::Write::flush(&mut io::stdout()).unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let kind_idx: usize = input.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
            let kind = MemoryKind::all().get(kind_idx).cloned().unwrap_or(MemoryKind::Convention);

            // Key
            print!("  {} ", "Key:".bold());
            io::Write::flush(&mut io::stdout()).unwrap();
            let mut key = String::new();
            std::io::stdin().read_line(&mut key).unwrap();
            let key = key.trim().to_string();

            // Content
            print!("  {} ", "Content:".bold());
            io::Write::flush(&mut io::stdout()).unwrap();
            let mut content = String::new();
            std::io::stdin().read_line(&mut content).unwrap();
            let content = content.trim().to_string();

            // Tags
            print!("  {} ", "Tags (comma-separated):".bold());
            io::Write::flush(&mut io::stdout()).unwrap();
            let mut tags_input = String::new();
            std::io::stdin().read_line(&mut tags_input).unwrap();
            let mut tags: Vec<String> = tags_input.trim().split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
            for dt in &config.default_tags {
                if !tags.contains(dt) {
                    tags.push(dt.clone());
                }
            }

            let memory = store
                .add(MemoryInput { kind, key, content, tags, related_ids: Vec::new() })
                .expect("Failed to add memory");

            println!();
            println!("{} Added memory {}", "[ok]".green(), memory.id[..8].to_string().dimmed());
            println!("  {} {}", "kind:".dimmed(), memory.kind);
            println!("  {} {}", "key:".dimmed(), memory.key.bold());
            println!("  {} {}", "content:".dimmed(), memory.content);
        }

        // --- Merge ---
        Command::Merge { other, strategy } => {
            let store = open_store(&project_dir);
            let other_store = MemoryStore::open_in_project(&other)
                .expect("Failed to open other project's memory store");

            let other_memories = other_store.export_json().expect("Failed to read other memories");
            let current = store.export_json().unwrap_or_default();
            let current_keys: std::collections::HashSet<String> = current.iter().map(|m| m.key.clone()).collect();

            let mut added = 0;
            let mut skipped = 0;

            for m in &other_memories {
                let exists = current_keys.contains(&m.key);
                match strategy.as_str() {
                    "skip" => {
                        if exists {
                            skipped += 1;
                            continue;
                        }
                    }
                    "overwrite"
                        if exists => {
                            // Delete existing with same key
                            if let Some(existing) = current.iter().find(|e| e.key == m.key) {
                                let _ = store.delete(&existing.id);
                            }
                        }
                    _ => {} // keep-both always adds
                }

                let _ = store.add(MemoryInput {
                    kind: m.kind.clone(),
                    key: m.key.clone(),
                    content: m.content.clone(),
                    tags: m.tags.clone(),
                    related_ids: m.related_ids.clone(),
                });
                added += 1;
            }

            println!("{} Merged memories from {}", "[ok]".green(), other.display());
            println!("  {} {}", "Added:".dimmed(), added);
            println!("  {} {}", "Skipped:".dimmed(), skipped);
        }

        // --- Validate ---
        Command::Validate { fix } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let mut issues = 0;

            println!("{}", "Validating memories...".bold());
            println!();

            for m in &memories {
                let mut mem_issues = Vec::new();

                // Check for empty content
                if m.content.trim().is_empty() {
                    mem_issues.push("empty content");
                }

                // Check for very short content
                if m.content.len() < 10 && !m.content.trim().is_empty() {
                    mem_issues.push("very short content (< 10 chars)");
                }

                // Check for duplicate tags
                let mut unique_tags = m.tags.clone();
                unique_tags.sort();
                unique_tags.dedup();
                if unique_tags.len() != m.tags.len() {
                    mem_issues.push("duplicate tags");
                }

                // Check for very long keys
                if m.key.len() > 50 {
                    mem_issues.push("key is very long (> 50 chars)");
                }

                // Check for broken related_ids
                for rid in &m.related_ids {
                    if store.get(rid).ok().flatten().is_none() {
                        mem_issues.push("broken related_id reference");
                    }
                }

                if !mem_issues.is_empty() {
                    issues += mem_issues.len();
                    println!("  {} {} ({})", "⚠".yellow(), m.key.bold(), &m.id[..8]);
                    for issue in &mem_issues {
                        println!("      {}", issue.dimmed());
                    }
                }
            }

            if issues == 0 {
                println!("{} All memories are valid!", "[ok]".green());
            } else {
                println!();
                println!("{} {} issues found in {} memories", "⚠".yellow(), issues, memories.len());
                if fix {
                    println!("  {} Auto-fix not yet implemented", "[i]".blue());
                }
            }
        }

        // --- Groups ---
        Command::Group { action } => match action {
            GroupAction::Create { name } => {
                let store = open_store(&project_dir);
                // Store group as a special context memory
                let memory = store.add(MemoryInput {
                    kind: MemoryKind::Context,
                    key: format!("group:{name}"),
                    content: format!("Memory group: {name}"),
                    tags: vec!["group".to_string()],
                    related_ids: Vec::new(),
                }).expect("Failed to create group");
                println!("{} Created group '{}'", "[ok]".green(), name);
                println!("  {} {}", "ID:".dimmed(), &memory.id[..8]);
            }
            GroupAction::List => {
                let store = open_store(&project_dir);
                let memories = store.export_json().unwrap_or_default();
                let groups: Vec<_> = memories.iter()
                    .filter(|m| m.tags.contains(&"group".to_string()) && m.key.starts_with("group:"))
                    .collect();

                if groups.is_empty() {
                    println!("{} No groups found. Create one with `pmem group create <name>`", "[i]".blue());
                    return;
                }

                println!("{} {} groups:\n", "📁".to_string().bold(), groups.len());
                for g in &groups {
                    let name = g.key.strip_prefix("group:").unwrap_or(&g.key);
                    let members: Vec<_> = memories.iter()
                        .filter(|m| m.related_ids.contains(&g.id))
                        .collect();
                    println!("  {} {} {}", name.bold(), format!("({} members)", members.len()).dimmed(), format!("({})", &g.id[..8]).dimmed());
                }
            }
            GroupAction::Add { group, id } => {
                let store = open_store(&project_dir);
                let memories = store.export_json().unwrap_or_default();
                let group_mem = memories.iter()
                    .find(|m| m.tags.contains(&"group".to_string()) && m.key == format!("group:{group}"));

                if let Some(g) = group_mem {
                    let id = resolve_id(&store, &id);
                    if store.link(&id, &g.id).expect("Failed to add to group") {
                        println!("{} Added {} to group '{}'", "[ok]".green(), &id[..8], group);
                    } else {
                        println!("{} Memory '{}' not found", "[x]".red(), id);
                    }
                } else {
                    println!("{} Group '{}' not found", "[x]".red(), group);
                }
            }
            GroupAction::Remove { group, id } => {
                let store = open_store(&project_dir);
                let memories = store.export_json().unwrap_or_default();
                let group_mem = memories.iter()
                    .find(|m| m.tags.contains(&"group".to_string()) && m.key == format!("group:{group}"));

                if let Some(g) = group_mem {
                    let id = resolve_id(&store, &id);
                    store.unlink(&id, &g.id).expect("Failed to remove from group");
                    println!("{} Removed {} from group '{}'", "[ok]".green(), &id[..8], group);
                } else {
                    println!("{} Group '{}' not found", "[x]".red(), group);
                }
            }
            GroupAction::Show { name } => {
                let store = open_store(&project_dir);
                let memories = store.export_json().unwrap_or_default();
                let group_mem = memories.iter()
                    .find(|m| m.tags.contains(&"group".to_string()) && m.key == format!("group:{name}"));

                if let Some(g) = group_mem {
                    let members: Vec<_> = memories.iter()
                        .filter(|m| m.related_ids.contains(&g.id))
                        .collect();

                    println!("{} Group '{}':\n", "📁".to_string().bold(), name);
                    if members.is_empty() {
                        println!("  {} No members", "[i]".blue());
                    } else {
                        for m in &members {
                            println!("  {} {} {}", format!("[{}]", m.kind).cyan(), m.key.bold(), format!("({})", &m.id[..8]).dimmed());
                            println!("    {}", m.content);
                        }
                    }
                } else {
                    println!("{} Group '{}' not found", "[x]".red(), name);
                }
            }
        },

        // --- Graph ---
        Command::Graph { format, output } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");

            match format.as_str() {
                "dot" => {
                    let path = output.unwrap_or_else(|| project_dir.join("memory-graph.dot"));
                    project_memory::graph::export_dot(&memories, &path).expect("Failed to export graph");
                    println!("{} Exported graph to {}", "[ok]".green(), path.display());
                    println!("  {} dot -Tpng {} -o graph.png", "Tip:".dimmed(), path.display());
                }
                "json" => {
                    let graph = project_memory::graph::export_json_graph(&memories);
                    if let Some(path) = output {
                        std::fs::write(&path, serde_json::to_string_pretty(&graph).unwrap()).expect("Failed to write");
                        println!("{} Exported graph to {}", "[ok]".green(), path.display());
                    } else {
                        println!("{}", serde_json::to_string_pretty(&graph).unwrap());
                    }
                }
                _ => {
                    println!("{} Unknown format '{}'. Use 'dot' or 'json'", "[x]".red(), format);
                }
            }
        }

        // --- Import Markdown ---
        Command::ImportMd { file } => {
            let store = open_store(&project_dir);
            let content = std::fs::read_to_string(&file).expect("Failed to read file");
            let inputs = project_memory::markdown::parse_markdown(&content);
            let count = store.import_memories(inputs).expect("Failed to import");
            println!("{} Imported {} memories from {}", "[ok]".green(), count, file.display());
        }

        // --- API Server ---
        Command::Api { port } => {
            println!("{} Starting REST API server on port {}...", "🌐".to_string().green(), port);
            println!("  {} http://{}/api/memories", "Endpoint:".dimmed(), port);
            println!("  {} http://{}/api/health", "Health:".dimmed(), port);
            project_memory::api::serve_api(project_dir, port).await;
        }

        // --- Pin ---
        Command::Pin { id } => {
            let store = open_store(&project_dir);
            let id = resolve_id(&store, &id);
            if store.pin(&id).expect("Failed to pin") {
                println!("{} Pinned memory {}", "[ok]".green(), &id[..8]);
            } else {
                println!("{} Memory '{}' not found", "[x]".red(), id);
            }
        }

        // --- Unpin ---
        Command::Unpin { id } => {
            let store = open_store(&project_dir);
            let id = resolve_id(&store, &id);
            if store.unpin(&id).expect("Failed to unpin") {
                println!("{} Unpinned memory {}", "[ok]".green(), &id[..8]);
            } else {
                println!("{} Memory '{}' not found", "[x]".red(), id);
            }
        }

        // --- Pinned ---
        Command::Pinned => {
            let store = open_store(&project_dir);
            let pinned = store.list_pinned().expect("Failed to list pinned");
            if pinned.is_empty() {
                println!("{} No pinned memories. Use `pmem pin <id>` to pin one", "[i]".blue());
                return;
            }
            println!("{} {} pinned memories:\n", " [PINNED]".to_string().bold(), pinned.len());
            for m in &pinned {
                println!("  {} {} {}", format!("[{}]", m.kind).cyan(), m.key.bold(), format!("({})", &m.id[..8]).dimmed());
                println!("    {}", m.content);
            }
        }

        // --- Date Search ---
        Command::DateSearch { after, before, limit } => {
            let store = open_store(&project_dir);
            let after_dt = after.and_then(|s| {
                chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .ok()
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                    .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc))
                    .or_else(|| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc)))
            });
            let before_dt = before.and_then(|s| {
                chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .ok()
                    .map(|d| d.and_hms_opt(23, 59, 59).unwrap())
                    .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc))
                    .or_else(|| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc)))
            });

            let memories = store.search_by_date(after_dt, before_dt, limit).expect("Failed to search");
            if memories.is_empty() {
                println!("{} No memories found in date range", "[i]".blue());
                return;
            }
            println!("{} {} memories in date range:\n", "📅".to_string().bold(), memories.len());
            for m in &memories {
                println!("  {} {} {} {}", format!("[{}]", m.kind).cyan(), m.key.bold(), m.updated_at.format("%Y-%m-%d").to_string().dimmed(), format!("({})", &m.id[..8]).dimmed());
                println!("    {}", m.content);
            }
        }

        // --- Hooks ---
        Command::Hooks { action } => {
            match action.unwrap_or(HooksAction::Status) {
                HooksAction::Install => {
                    println!("{}", "Installing git hooks...".bold());
                    project_memory::hooks::install_hooks(&project_dir).expect("Failed to install hooks");
                    println!("{} Hooks installed", "\u{2713}".green());
                }
                HooksAction::Uninstall => {
                    project_memory::hooks::uninstall_hooks(&project_dir).expect("Failed to uninstall hooks");
                    println!("{} Hooks uninstalled", "\u{2713}".green());
                }
                HooksAction::Status => {
                    project_memory::hooks::hooks_status(&project_dir).expect("Failed to check hooks");
                }
            }
        }

        // --- Import Code Comments ---
        Command::ImportCode { dir, extensions } => {
            let store = open_store(&project_dir);
            let scan_dir = dir.map(|d| project_dir.join(d)).unwrap_or_else(|| project_dir.clone());
            let exts: Vec<&str> = extensions.as_deref()
                .unwrap_or("rs,ts,js,py,go")
                .split(',')
                .collect();
            let memories = project_memory::code_import::scan_directory(&scan_dir, &exts)
                .expect("Failed to scan directory");
            let count = store.import_memories(memories).expect("Failed to import");
            println!("{} Imported {} code comments as memories", "\u{2713}".green(), count);
        }

        // --- Encryption ---
        Command::Encryption { action } => {
            match action {
                EncryptionAction::SetKey { key } => {
                    project_memory::encryption::set_encryption_key(&project_dir, &key)
                        .expect("Failed to set key");
                    println!("{} Encryption key set", "\u{2713}".green());
                }
                EncryptionAction::RemoveKey => {
                    project_memory::encryption::remove_encryption_key(&project_dir)
                        .expect("Failed to remove key");
                    println!("{} Encryption key removed", "\u{2713}".green());
                }
                EncryptionAction::Status => {
                    let key = project_memory::encryption::get_encryption_key(&project_dir);
                    match key {
                        Some(_) => println!("{} Encryption is enabled", "\u{1f512}".to_string().green()),
                        None => println!("{} Encryption is disabled", "\u{1f513}".to_string().dimmed()),
                    }
                }
                EncryptionAction::Encrypt { id } => {
                    let store = open_store(&project_dir);
                    let key = project_memory::encryption::get_encryption_key(&project_dir)
                        .expect("No encryption key set");
                    let id = resolve_id(&store, &id);
                    if let Some(m) = store.get(&id).expect("Failed to get memory") {
                        if m.content.starts_with("ENC:") {
                            println!("{} Memory is already encrypted", "[i]".blue());
                            return;
                        }
                        let encrypted_content = project_memory::encryption::encrypt(&m.content, &key);
                        let _ = store.update(&id, project_memory::types::MemoryInput {
                            kind: m.kind,
                            key: m.key,
                            content: format!("ENC:{}", encrypted_content),
                            tags: m.tags,
                            related_ids: m.related_ids,
                        });
                        println!("{} Memory encrypted", "\u{2713}".green());
                    } else {
                        println!("{} Memory not found", "\u{2717}".red());
                    }
                }
                EncryptionAction::Decrypt { id } => {
                    let store = open_store(&project_dir);
                    let key = project_memory::encryption::get_encryption_key(&project_dir)
                        .expect("No encryption key set");
                    let id = resolve_id(&store, &id);
                    if let Some(m) = store.get(&id).expect("Failed to get memory") {
                        if let Some(enc) = m.content.strip_prefix("ENC:") {
                            let decrypted = project_memory::encryption::decrypt(enc, &key)
                                .expect("Failed to decrypt");
                            println!("{} Decrypted content:", "\u{1f513}".to_string().bold());
                            println!("  {}", decrypted);
                        } else {
                            println!("{} Memory is not encrypted", "\u{2139}".blue());
                        }
                    } else {
                        println!("{} Memory not found", "\u{2717}".red());
                    }
                }
            }
        }



        // --- Analytics ---
        Command::Analytics { format } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let analytics = project_memory::analytics::compute_analytics(&memories);

            match format.as_deref().unwrap_or("text") {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&analytics).unwrap());
                }
                _ => {
                    println!("{}", project_memory::analytics::format_analytics(&analytics));
                }
            }
        }

        // --- Docs ---
        Command::Docs { format } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let fmt = format.as_deref().unwrap_or("markdown");

            match project_memory::docs::save_documentation(&project_dir, &memories, fmt) {
                Ok(path) => {
                    println!("{} Generated documentation: {}", "[ok]".green(), path);
                }
                Err(e) => {
                    println!("{} Failed to generate docs: {}", "[x]".red(), e);
                }
            }
        }

        // --- Export ---
        Command::ExportFormat { format } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let fmt = format.as_deref().unwrap_or("json");

            match project_memory::export::save_export(&project_dir, &memories, fmt) {
                Ok(path) => {
                    println!("{} Exported to: {}", "[ok]".green(), path);
                }
                Err(e) => {
                    println!("{} Failed to export: {}", "[x]".red(), e);
                }
            }
        }

        // --- Backup ---
        Command::Backup => {
            let _store = open_store(&project_dir);
            match project_memory::backup::create_backup(&project_dir) {
                Ok(path) => {
                    println!("{} Backup created: {}", "[ok]".green(), path);
                }
                Err(e) => {
                    println!("{} Failed to create backup: {}", "[x]".red(), e);
                }
            }
        }

        // --- Backup List ---
        Command::BackupList => {
            match project_memory::backup::list_backups(&project_dir) {
                Ok(backups) => {
                    if backups.is_empty() {
                        println!("{} No backups found", "[i]".blue());
                        return;
                    }
                    println!("{} {} backups:\n", ">>".to_string().bold(), backups.len());
                    for b in &backups {
                        println!("  {} {} ({} KB)", b.name.bold(), b.created.format("%Y-%m-%d %H:%M").to_string().dimmed(), b.size / 1024);
                    }
                }
                Err(e) => {
                    println!("{} Failed to list backups: {}", "[x]".red(), e);
                }
            }
        }

        // --- Backup Restore ---
        Command::BackupRestore { name } => {
            match project_memory::backup::restore_backup(&project_dir, &name) {
                Ok(()) => {
                    println!("{} Restored backup: {}", "[ok]".green(), name);
                }
                Err(e) => {
                    println!("{} Failed to restore backup: {}", "[x]".red(), e);
                }
            }
        }

        // --- Backup Cleanup ---
        Command::BackupCleanup { keep } => {
            match project_memory::backup::cleanup_backups(&project_dir, keep) {
                Ok(deleted) => {
                    println!("{} Cleaned up {} old backups", "[ok]".green(), deleted);
                }
                Err(e) => {
                    println!("{} Failed to cleanup backups: {}", "[x]".red(), e);
                }
            }
        }

        // --- Migrate ---
        Command::Migrate { source, tool } => {
            let store = open_store(&project_dir);
            let source_path = std::path::PathBuf::from(&source);
            
            let memories = match tool.as_deref().unwrap_or("auto") {
                "obsidian" => {
                    project_memory::migration::import_from_obsidian(&source_path)
                }
                "notion" => {
                    project_memory::migration::import_from_notion_csv(&source_path)
                }
                "json" => {
                    project_memory::migration::import_from_json_file(&source_path)
                }
                "markdown" | "md" => {
                    project_memory::migration::import_from_markdown_dir(&source_path)
                }
                "auto" => {
                    // Try to detect based on file extension
                    if source_path.extension().is_some_and(|e| e == "json") {
                        project_memory::migration::import_from_json_file(&source_path)
                    } else if source_path.extension().is_some_and(|e| e == "csv") {
                        project_memory::migration::import_from_notion_csv(&source_path)
                    } else if source_path.is_dir() {
                        project_memory::migration::import_from_markdown_dir(&source_path)
                    } else {
                        Err(anyhow::anyhow!("Cannot auto-detect format. Use --tool to specify."))
                    }
                }
                _ => Err(anyhow::anyhow!("Unknown tool"))
            };
            
            match memories {
                Ok(inputs) => {
                    let count = store.import_memories(inputs).expect("Failed to import");
                    println!("{} Imported {} memories", "[ok]".green(), count);
                }
                Err(e) => {
                    println!("{} Failed to migrate: {}", "[x]".red(), e);
                }
            }
        }

        // --- Graph Stats ---
        Command::GraphStats => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let graph = project_memory::graph_query::MemoryGraph::from_memories(&memories);
            println!("{}", project_memory::graph_query::format_graph_stats(&graph));
        }

        // --- Graph Path ---
        Command::GraphPath { from, to } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let graph = project_memory::graph_query::MemoryGraph::from_memories(&memories);
            
            let from_id = resolve_id(&store, &from);
            let to_id = resolve_id(&store, &to);
            
            match graph.shortest_path(&from_id, &to_id) {
                Some(path) => {
                    println!("{} Path ({} steps):", ">>".to_string().bold(), path.len() - 1);
                    for (i, id) in path.iter().enumerate() {
                        if let Ok(Some(m)) = store.get(id) {
                            if i > 0 {
                                println!("  {}", "|".dimmed());
                            }
                            println!("  {} {} {}", format!("[{}]", m.kind).cyan(), m.key.bold(), format!("({})", &id[..8]).dimmed());
                        }
                    }
                }
                None => {
                    println!("{} No path found between these memories", "[x]".red());
                }
            }
        }

        // --- Graph Components ---
        Command::GraphComponents => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let graph = project_memory::graph_query::MemoryGraph::from_memories(&memories);
            let components = graph.connected_components();
            
            println!("{} {} connected components:
", ">>".to_string().bold(), components.len());
            for (i, component) in components.iter().enumerate() {
                println!("  Component {} ({} nodes):", i + 1, component.len());
                for id in component.iter().take(3) {
                    if let Ok(Some(m)) = store.get(id) {
                        println!("    - {} {}", m.key, format!("({})", &id[..8]).dimmed());
                    }
                }
                if component.len() > 3 {
                    println!("    ... and {} more", component.len() - 3);
                }
            }
        }

        // --- FTS Search ---
        Command::FtsSearch { query, limit } => {
            let store = open_store(&project_dir);
            match project_memory::fts::fts_search(store.conn(), &query, limit) {
                Ok(results) => {
                    if results.is_empty() {
                        println!("{} No results for '{}'", "[i]".blue(), query);
                    } else {
                        println!("{}", project_memory::fts::format_fts_results(&results));
                    }
                }
                Err(e) => {
                    println!("{} FTS search failed: {}", "[x]".red(), e);
                    println!("  {} Try using regular search: pmem search <query>", "Tip:".dimmed());
                }
            }
        }

        // --- FTS Rebuild ---
        Command::FtsRebuild => {
            let store = open_store(&project_dir);
            match project_memory::fts::rebuild_fts(store.conn()) {
                Ok(()) => {
                    println!("{} FTS index rebuilt", "[ok]".green());
                }
                Err(e) => {
                    println!("{} Failed to rebuild FTS: {}", "[x]".red(), e);
                }
            }
        }

        // --- Suggestions ---
        Command::Suggestions { query } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let suggestions = project_memory::suggestions::get_suggestions(&memories, &query);
            
            if suggestions.is_empty() {
                println!("{} No suggestions for '{}'", "[i]".blue(), query);
            } else {
                println!("{}", project_memory::suggestions::format_suggestions(&suggestions));
            }
        }

        // --- Popular Searches ---
        Command::PopularSearches => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let popular = project_memory::suggestions::get_popular_searches(&memories);
            
            if popular.is_empty() {
                println!("{} No popular searches yet", "[i]".blue());
            } else {
                println!("{} Popular search terms:
", ">>".to_string().bold());
                for (i, term) in popular.iter().enumerate() {
                    println!("  {}. {}", i + 1, term);
                }
            }
        }

        // --- Quick Start ---
        Command::QuickStart => {
            project_memory::help::show_quick_start(&project_dir);
        }

        // --- Tips ---
        Command::Tips => {
            project_memory::help::show_tips();
        }

        // --- Highlight Search ---
        Command::HighlightSearch { query } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let results = project_memory::highlight::highlight_search(&memories, &query);

            if results.is_empty() {
                println!("{} No memories match '{}'", "[i]".blue(), query);
                return;
            }
            println!("{} {} results for '{}'\n", "[i]".blue(), results.len(), query);
            for r in &results {
                print!("{}", project_memory::highlight::format_highlighted(r));
            }
        }

        // --- Validate Improved ---
        Command::ValidateImproved { fix } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export");
            let result = project_memory::validation::validate_memories(&memories);
            
            println!("{}", project_memory::validation::format_validation(&result));
            
            if fix && result.auto_fixable > 0 {
                let mut memories = store.export_json().expect("Failed to export");
                let fixed_ids = project_memory::validation::auto_fix(&mut memories);

                for m in memories.iter().filter(|m| fixed_ids.contains(&m.id)) {
                    let _ = store.update(&m.id, project_memory::types::MemoryInput {
                        kind: m.kind.clone(),
                        key: m.key.clone(),
                        content: m.content.clone(),
                        tags: m.tags.clone(),
                        related_ids: m.related_ids.clone(),
                    });
                }

                println!("{} Auto-fixed {} memories", "[ok]".green(), fixed_ids.len());
            }
        }

        // --- Memory Stats Improved ---
        Command::StatsImproved => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export");
            let result = project_memory::validation::validate_memories(&memories);
            let analytics = project_memory::analytics::compute_analytics(&memories);
            
            println!("{}", project_memory::analytics::format_analytics(&analytics));
            println!();
            println!("{}", project_memory::validation::format_validation(&result));
        }

        // --- Faceted Search ---
        Command::FacetedSearch { query, kind, tags, pinned, limit } => {
            let store = open_store(&project_dir);
            let kind_parsed = kind.and_then(|k| k.parse::<project_memory::types::MemoryKind>().ok());
            let tag_list: Vec<String> = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default();
            let limit = limit.unwrap_or(20);

            let memories = store.faceted_search(
                query.as_deref(),
                kind_parsed.as_ref(),
                &tag_list,
                pinned,
                limit,
            ).expect("Failed to search");

            if memories.is_empty() {
                println!("{} No memories match your filters", "[i]".blue());
                return;
            }
            println!("{} {} results\n", "[i]".blue(), memories.len());
            for m in &memories {
                println!("  {} {} {}", format!("[{}]", m.kind).cyan(), m.key.bold(), format!("({})", &m.id[..8]).dimmed());
                println!("    {}", m.content);
            }
        }

        // --- Plugin Management ---
        Command::Plugin { action } => {
            match action {
                PluginAction::List => {
                    let plugins = project_memory::plugin::list_plugins(&project_dir);
                    if plugins.is_empty() {
                        println!("{} No plugins installed", "[i]".blue());
                        println!("  Create a plugin in .memory/plugins/");
                    } else {
                        println!("{} {} plugins:\n", ">>".to_string().bold(), plugins.len());
                        for p in &plugins {
                            let status = if p.enabled { "enabled" } else { "disabled" };
                            println!("  {} v{} ({})", p.name.bold(), p.version, status);
                            println!("    {}", p.description);
                        }
                    }
                }
                PluginAction::Create => {
                    let plugin = project_memory::plugin::create_example_plugin();
                    project_memory::plugin::save_plugin(&project_dir, &plugin).expect("Failed to save plugin");
                    println!("{} Created example plugin in .memory/plugins/", "[ok]".green());
                }
                PluginAction::Enable { name } => {
                    let mut plugins = project_memory::plugin::list_plugins(&project_dir);
                    if let Some(p) = plugins.iter_mut().find(|p| p.name == name) {
                        p.enabled = true;
                        project_memory::plugin::save_plugin(&project_dir, p).expect("Failed to save plugin");
                        println!("{} Enabled plugin '{}'", "[ok]".green(), name);
                    } else {
                        println!("{} Plugin '{}' not found", "[x]".red(), name);
                    }
                }
                PluginAction::Disable { name } => {
                    let mut plugins = project_memory::plugin::list_plugins(&project_dir);
                    if let Some(p) = plugins.iter_mut().find(|p| p.name == name) {
                        p.enabled = false;
                        project_memory::plugin::save_plugin(&project_dir, p).expect("Failed to save plugin");
                        println!("{} Disabled plugin '{}'", "[ok]".green(), name);
                    } else {
                        println!("{} Plugin '{}' not found", "[x]".red(), name);
                    }
                }
                PluginAction::Remove { name } => {
                    if project_memory::plugin::remove_plugin(&project_dir, &name).expect("Failed to remove plugin") {
                        println!("{} Removed plugin '{}'", "[ok]".green(), name);
                    } else {
                        println!("{} Plugin '{}' not found", "[x]".red(), name);
                    }
                }
            }
        }

        // --- Webhook Management ---
        Command::Webhook { action } => {
            match action {
                WebhookAction::List => {
                    let webhooks = project_memory::webhook::list_webhooks(&project_dir);
                    if webhooks.is_empty() {
                        println!("{} No webhooks configured", "[i]".blue());
                    } else {
                        println!("{} {} webhooks:\n", ">>".to_string().bold(), webhooks.len());
                        for w in &webhooks {
                            let status = if w.enabled { "enabled" } else { "disabled" };
                            println!("  {} ({})", w.name.bold(), status);
                            println!("    URL: {}", w.url);
                            println!("    Events: {}", w.events.join(", "));
                        }
                    }
                }
                WebhookAction::Add { name, url, events } => {
                    let events: Vec<String> = events.split(',').map(|e| e.trim().to_string()).collect();
                    let webhook = project_memory::webhook::Webhook {
                        name: name.clone(),
                        url,
                        events,
                        secret: None,
                        enabled: true,
                    };
                    project_memory::webhook::add_webhook(&project_dir, webhook).expect("Failed to add webhook");
                    println!("{} Added webhook '{}'", "[ok]".green(), name);
                }
                WebhookAction::Remove { name } => {
                    if project_memory::webhook::remove_webhook(&project_dir, &name).expect("Failed to remove webhook") {
                        println!("{} Removed webhook '{}'", "[ok]".green(), name);
                    } else {
                        println!("{} Webhook '{}' not found", "[x]".red(), name);
                    }
                }
            }
        }

        // --- Sync ---
        Command::Sync { remote, strategy } => {
            let remote_path = std::path::PathBuf::from(&remote);
            match project_memory::sync::sync_memories(&project_dir, &remote_path, &strategy) {
                Ok(state) => {
                    println!("{} Sync complete", "[ok]".green());
                    println!("  Synced: {} memories", state.memories_synced);
                    if !state.conflicts.is_empty() {
                        println!("  Conflicts: {}", state.conflicts.len());
                        for c in &state.conflicts {
                            println!("    - {}", c);
                        }
                    }
                }
                Err(e) => {
                    println!("{} Sync failed: {}", "[x]".red(), e);
                }
            }
        }

        // --- Sync Export ---
        Command::SyncExport => {
            match project_memory::sync::export_sync_bundle(&project_dir) {
                Ok(bundle) => {
                    let path = project_dir.join(".memory").join("sync_bundle.json");
                    std::fs::write(&path, &bundle).expect("Failed to write bundle");
                    println!("{} Exported sync bundle to {}", "[ok]".green(), path.display());
                }
                Err(e) => {
                    println!("{} Export failed: {}", "[x]".red(), e);
                }
            }
        }

        // --- Sync Import ---
        Command::SyncImport { file, strategy } => {
            let content = std::fs::read_to_string(&file).expect("Failed to read file");
            match project_memory::sync::import_sync_bundle(&project_dir, &content, &strategy) {
                Ok(count) => {
                    println!("{} Imported {} memories", "[ok]".green(), count);
                }
                Err(e) => {
                    println!("{} Import failed: {}", "[x]".red(), e);
                }
            }
        }

        // --- Analytics V2 ---
        Command::AnalyticsV2 => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export memories");
            let analytics = project_memory::analytics_v2::compute_analytics_v2(&memories);
            println!("{}", project_memory::analytics_v2::format_analytics_v2(&analytics));
        }

        // --- Custom Templates ---
        Command::CustomTemplate { action } => {
            match action {
                CustomTemplateAction::List => {
                    let templates = project_memory::custom_templates::list_custom_templates(&project_dir);
                    if templates.is_empty() {
                        println!("{} No custom templates", "[i]".blue());
                        println!("  Create one with: pmem custom-template create <name>");
                    } else {
                        println!("{} {} custom templates:\n", ">>".to_string().bold(), templates.len());
                        for (name, desc, count) in &templates {
                            println!("  {} ({} memories)", name.bold(), count);
                            println!("    {}", desc);
                        }
                    }
                }
                CustomTemplateAction::Create { name, description } => {
                    let store = open_store(&project_dir);
                    let memories = store.export_json().expect("Failed to export memories");
                    let template = project_memory::custom_templates::create_template_from_memories(&name, &description, &memories);
                    project_memory::custom_templates::save_custom_template(&project_dir, &template).expect("Failed to save template");
                    println!("{} Created custom template '{}' with {} memories", "[ok]".green(), name, memories.len());
                }
                CustomTemplateAction::Apply { name } => {
                    match project_memory::custom_templates::apply_custom_template(&project_dir, &name) {
                        Ok(count) => {
                            println!("{} Applied template '{}' - {} memories added", "[ok]".green(), name, count);
                        }
                        Err(e) => {
                            println!("{} Failed to apply template: {}", "[x]".red(), e);
                        }
                    }
                }
                CustomTemplateAction::Delete { name } => {
                    if project_memory::custom_templates::delete_custom_template(&project_dir, &name).expect("Failed to delete template") {
                        println!("{} Deleted template '{}'", "[ok]".green(), name);
                    } else {
                        println!("{} Template '{}' not found", "[x]".red(), name);
                    }
                }
            }
        }

        // --- Watch ---
        Command::Watch { interval } => {
            let config = project_memory::watch::WatchConfig {
                interval: std::time::Duration::from_secs(interval.unwrap_or(30)),
                auto_sync: false,
                auto_backup: false,
                notify: true,
            };
            println!("Watching for changes (interval: {}s)", config.interval.as_secs());
            println!("Press Ctrl+C to stop");
            loop {
                std::thread::sleep(config.interval);
                let store = open_store(&project_dir);
                let count = store.count().unwrap_or(0);
                println!("  [{}] {} memories", chrono::Utc::now().format("%H:%M:%S"), count);
            }
        }

        // --- Watch Status ---
        Command::WatchStatus => {
            match project_memory::watch::watch_status(&project_dir) {
                Ok(status) => {
                    println!("{}", project_memory::watch::format_watch_status(&status));
                }
                Err(e) => {
                    println!("{} Failed to get watch status: {}", "[x]".red(), e);
                }
            }
        }

        // --- Advanced Search ---
        Command::AdvancedSearch { query, kind, tag, pinned, limit } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export");
            
            let filters = project_memory::search_v2::SearchFilters {
                kind: kind,
                tag: tag,
                pinned_only: pinned,
                limit: limit.unwrap_or(20),
            };
            
            let results = project_memory::search_v2::advanced_search(&memories, &query, &filters);
            println!("{}", project_memory::search_v2::format_search_results(&results));
        }

        // --- Search Suggestions ---
        Command::SearchSuggest { query } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export");
            let suggestions = project_memory::search_v2::search_suggestions(&memories, &query);
            
            if suggestions.is_empty() {
                println!("{} No suggestions for '{}'", "[i]".blue(), query);
            } else {
                println!("Suggestions for '{}':", query);
                for s in &suggestions {
                    println!("  {}", s);
                }
            }
        }

        // --- Export HTML ---
        Command::ExportHtml { output } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export");
            let html = project_memory::export_v2::export_html_report(&memories, "Project Memory");
            
            let path = output.unwrap_or_else(|| project_dir.join("memory-report.html"));
            std::fs::write(&path, html).expect("Failed to write file");
            println!("{} Exported HTML report to {}", "[ok]".green(), path.display());
        }

        // --- Export Schema ---
        Command::ExportSchema => {
            let schema = project_memory::export_v2::export_json_schema();
            println!("{}", schema);
        }

        // --- Validate V2 ---
        Command::ValidateV2 { fix } => {
            let store = open_store(&project_dir);
            let memories = store.export_json().expect("Failed to export");
            let report = project_memory::validation_v2::validate_v2(&memories);
            
            println!("{}", project_memory::validation_v2::format_validation_report(&report));
            
            if fix && report.auto_fixable > 0 {
                let mut memories = store.export_json().expect("Failed to export");
                let fixed = project_memory::validation_v2::auto_fix_v2(&mut memories);
                
                for m in &memories {
                    let _ = store.update(&m.id, project_memory::types::MemoryInput {
                        kind: m.kind.clone(),
                        key: m.key.clone(),
                        content: m.content.clone(),
                        tags: m.tags.clone(),
                        related_ids: m.related_ids.clone(),
                    });
                }
                
                println!("{} Auto-fixed {} memories", "[ok]".green(), fixed);
            }
        }

    }
}
