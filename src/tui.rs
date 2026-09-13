use crate::store::MemoryStore;
use crate::types::Memory;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;

struct App {
    memories: Vec<Memory>,
    list_state: ListState,
    search_input: String,
    search_mode: bool,
    filtered_indices: Vec<usize>,
    detail_scroll: u16,
}

impl App {
    fn new(store: &MemoryStore) -> Self {
        let memories = store.list(None, 10000).unwrap_or_default();
        let filtered_indices: Vec<usize> = (0..memories.len()).collect();
        let mut list_state = ListState::default();
        if !memories.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            memories,
            list_state,
            search_input: String::new(),
            search_mode: false,
            filtered_indices,
            detail_scroll: 0,
        }
    }

    fn selected_memory(&self) -> Option<&Memory> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i))
            .and_then(|&idx| self.memories.get(idx))
    }

    fn apply_filter(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_indices = (0..self.memories.len()).collect();
        } else {
            let query = self.search_input.to_lowercase();
            self.filtered_indices = self
                .memories
                .iter()
                .enumerate()
                .filter(|(_, m)| {
                    m.key.to_lowercase().contains(&query)
                        || m.content.to_lowercase().contains(&query)
                        || m.tags.iter().any(|t| t.to_lowercase().contains(&query))
                        || m.kind.to_string().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.list_state.select(
            if self.filtered_indices.is_empty() {
                None
            } else {
                Some(0)
            },
        );
    }
}

pub fn run_tui(project_dir: PathBuf) -> anyhow::Result<()> {
    let store = MemoryStore::open_in_project(&project_dir)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&store);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app.search_mode {
                    match key.code {
                        KeyCode::Esc => {
                            app.search_mode = false;
                            app.search_input.clear();
                            app.apply_filter();
                        }
                        KeyCode::Enter => {
                            app.search_mode = false;
                        }
                        KeyCode::Backspace => {
                            app.search_input.pop();
                            app.apply_filter();
                        }
                        KeyCode::Char(c) => {
                            app.search_input.push(c);
                            app.apply_filter();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('/') => {
                            app.search_mode = true;
                            app.search_input.clear();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.list_state.select(Some(
                                app.list_state
                                    .selected()
                                    .map(|i| {
                                        if i + 1 < app.filtered_indices.len() {
                                            i + 1
                                        } else {
                                            i
                                        }
                                    })
                                    .unwrap_or(0),
                            ));
                            app.detail_scroll = 0;
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.list_state.select(Some(
                                app.list_state
                                    .selected()
                                    .map(|i| i.saturating_sub(1))
                                    .unwrap_or(0),
                            ));
                            app.detail_scroll = 0;
                        }
                        KeyCode::Char('g') => {
                            if !app.filtered_indices.is_empty() {
                                app.list_state.select(Some(0));
                            }
                        }
                        KeyCode::Char('G') => {
                            if !app.filtered_indices.is_empty() {
                                app.list_state.select(Some(app.filtered_indices.len() - 1));
                            }
                        }
                        KeyCode::Char('d') => {
                            app.detail_scroll = app.detail_scroll.saturating_add(3);
                        }
                        KeyCode::Char('u') => {
                            app.detail_scroll = app.detail_scroll.saturating_sub(3);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(f.area());

    // Left panel - list
    let list_items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .filter_map(|&idx| app.memories.get(idx))
        .map(|m| {
            let kind_style = match m.kind {
                crate::types::MemoryKind::Convention => Style::default().fg(Color::Cyan),
                crate::types::MemoryKind::Pattern => Style::default().fg(Color::Green),
                crate::types::MemoryKind::Decision => Style::default().fg(Color::Yellow),
                crate::types::MemoryKind::Preference => Style::default().fg(Color::Magenta),
                crate::types::MemoryKind::Context => Style::default().fg(Color::Blue),
            };

            let line = Line::from(vec![
                Span::styled(format!("[{}] ", m.kind), kind_style),
                Span::styled(&m.key, Style::default().add_modifier(Modifier::BOLD)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let search_hint = if app.search_mode {
        format!(" Search: {}_", app.search_input)
    } else {
        format!(
            " {}/{} memories (press / to search)",
            app.filtered_indices.len(),
            app.memories.len()
        )
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!("project-memory{search_hint}"))
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.list_state);

    // Right panel - detail
    let detail = if let Some(m) = app.selected_memory() {
        let kind_style = match m.kind {
            crate::types::MemoryKind::Convention => Style::default().fg(Color::Cyan),
            crate::types::MemoryKind::Pattern => Style::default().fg(Color::Green),
            crate::types::MemoryKind::Decision => Style::default().fg(Color::Yellow),
            crate::types::MemoryKind::Preference => Style::default().fg(Color::Magenta),
            crate::types::MemoryKind::Context => Style::default().fg(Color::Blue),
        };

        let mut lines = vec![
            Line::from(vec![
                Span::raw("Kind:    "),
                Span::styled(m.kind.to_string(), kind_style),
            ]),
            Line::from(vec![
                Span::raw("Key:     "),
                Span::styled(&m.key, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Content:", Style::default().add_modifier(Modifier::UNDERLINED))),
        ];

        for content_line in m.content.lines() {
            lines.push(Line::from(content_line.to_string()));
        }

        lines.push(Line::from(""));

        if !m.tags.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("Tags:    "),
                Span::styled(m.tags.join(", "), Style::default().fg(Color::Yellow)),
            ]));
        }

        if !m.related_ids.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Related memories:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            for rid in &m.related_ids {
                lines.push(Line::from(format!("  - {}", &rid[..8.min(rid.len())])));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("ID:      "),
            Span::styled(&m.id[..12], Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("Created: "),
            Span::styled(m.created_at.format("%Y-%m-%d %H:%M").to_string(), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("Updated: "),
            Span::styled(m.updated_at.format("%Y-%m-%d %H:%M").to_string(), Style::default().fg(Color::DarkGray)),
        ]));

        Paragraph::new(lines)
            .block(Block::default().title(" Details ").borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .scroll((app.detail_scroll, 0))
    } else {
        Paragraph::new("No memory selected")
            .block(Block::default().title(" Details ").borders(Borders::ALL))
    };

    f.render_widget(detail, chunks[1]);
}
