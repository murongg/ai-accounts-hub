use std::io::{self, Stdout};
use std::time::Duration;

use aah_core::cli_facade::{AccountRow, CliFacade, CurrentRow, Provider, SwitchSelection};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend, TestBackend};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::{Frame, Terminal};

pub fn run_tui(facade: &CliFacade, snapshot: bool) -> Result<(), String> {
    let model = TuiModel::from_facade(facade)?;
    if snapshot {
        println!("{}", render_snapshot(&model)?);
        return Ok(());
    }

    let mut terminal = enter_terminal()?;
    let result = run_event_loop(&mut terminal, facade, model);
    let exit_result = exit_terminal(&mut terminal);
    result.and(exit_result)
}

struct TuiModel {
    accounts: Vec<AccountRow>,
    current: Vec<CurrentRow>,
    filter: Option<Provider>,
    selected: usize,
    status: String,
}

impl TuiModel {
    fn from_facade(facade: &CliFacade) -> Result<Self, String> {
        let mut model = Self {
            accounts: Vec::new(),
            current: Vec::new(),
            filter: None,
            selected: 0,
            status: "Ready".to_string(),
        };
        model.reload(facade)?;
        Ok(model)
    }

    fn reload(&mut self, facade: &CliFacade) -> Result<(), String> {
        self.accounts = facade
            .list(self.filter)
            .map_err(|error| error.to_string())?;
        self.current = facade
            .current(self.filter)
            .map_err(|error| error.to_string())?;
        self.clamp_selection();
        Ok(())
    }

    fn clamp_selection(&mut self) {
        if self.accounts.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.accounts.len() {
            self.selected = self.accounts.len() - 1;
        }
    }

    fn handle_key(&mut self, facade: &CliFacade, key: KeyEvent) -> Result<bool, String> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(true),
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                Ok(false)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                Ok(false)
            }
            KeyCode::Enter => {
                self.switch_selected(facade);
                Ok(false)
            }
            KeyCode::Char('r') => {
                self.refresh(facade)?;
                Ok(false)
            }
            KeyCode::Char('1') => self.set_filter(facade, Some(Provider::Codex)),
            KeyCode::Char('2') => self.set_filter(facade, Some(Provider::Claude)),
            KeyCode::Char('3') => self.set_filter(facade, Some(Provider::Gemini)),
            KeyCode::Char('a') => self.set_filter(facade, None),
            _ => Ok(false),
        }
    }

    fn select_next(&mut self) {
        if self.accounts.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.accounts.len();
    }

    fn select_previous(&mut self) {
        if self.accounts.is_empty() {
            return;
        }
        self.selected = if self.selected == 0 {
            self.accounts.len() - 1
        } else {
            self.selected - 1
        };
    }

    fn switch_selected(&mut self, facade: &CliFacade) {
        let Some(account) = self.accounts.get(self.selected).cloned() else {
            self.status = "No account selected".to_string();
            return;
        };

        match facade.switch(account.provider, SwitchSelection::Id(account.id.clone())) {
            Ok(outcome) => {
                self.status = format!(
                    "{} {} account: {}",
                    if outcome.already_active {
                        "Re-synced"
                    } else {
                        "Switched"
                    },
                    provider_title(outcome.provider),
                    outcome.email
                );
                if let Err(error) = self.reload(facade) {
                    self.status = format!("Reload failed after switch: {error}");
                }
            }
            Err(error) => {
                self.status = format!("Switch failed: {error}");
            }
        }
    }

    fn refresh(&mut self, facade: &CliFacade) -> Result<(), String> {
        let rows = facade
            .refresh(self.filter)
            .map_err(|error| error.to_string())?;
        let failed = rows.iter().filter(|row| !row.ok).count();
        self.status = if failed == 0 {
            format!("Refreshed {}", provider_filter_label(self.filter))
        } else {
            format!("Refresh finished with {failed} provider error(s)")
        };
        self.reload(facade)
    }

    fn set_filter(&mut self, facade: &CliFacade, filter: Option<Provider>) -> Result<bool, String> {
        self.filter = filter;
        self.selected = 0;
        self.reload(facade)?;
        self.status = format!("Filter: {}", provider_filter_label(filter));
        Ok(false)
    }
}

fn enter_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, String> {
    enable_raw_mode().map_err(|error| error.to_string())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|error| error.to_string())?;
    Terminal::new(CrosstermBackend::new(stdout)).map_err(|error| error.to_string())
}

fn exit_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), String> {
    disable_raw_mode().map_err(|error| error.to_string())?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|error| error.to_string())?;
    terminal.show_cursor().map_err(|error| error.to_string())
}

fn run_event_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    facade: &CliFacade,
    mut model: TuiModel,
) -> Result<(), String> {
    loop {
        terminal
            .draw(|frame| render(frame, &model))
            .map_err(|error| error.to_string())?;

        if !event::poll(Duration::from_millis(250)).map_err(|error| error.to_string())? {
            continue;
        }

        if let Event::Key(key) = event::read().map_err(|error| error.to_string())? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            if model.handle_key(facade, key)? {
                return Ok(());
            }
        }
    }
}

fn render_snapshot(model: &TuiModel) -> Result<String, String> {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).map_err(|error| error.to_string())?;
    terminal
        .draw(|frame| render(frame, model))
        .map_err(|error| error.to_string())?;
    Ok(format!("{:?}", terminal.backend().buffer()))
}

fn render(frame: &mut Frame<'_>, model: &TuiModel) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Min(9),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_header(frame, chunks[0], model);
    render_current(frame, chunks[1], &model.current, model.filter);
    render_accounts(frame, chunks[2], &model.accounts, model.selected);
    render_footer(frame, chunks[3], &model.status);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, model: &TuiModel) {
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "AI Accounts Hub",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!(
            "Terminal account switchboard | filter: {} | accounts: {}",
            provider_filter_label(model.filter),
            model.accounts.len()
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title("aah tui"));
    frame.render_widget(header, area);
}

fn render_current(
    frame: &mut Frame<'_>,
    area: Rect,
    current: &[CurrentRow],
    filter: Option<Provider>,
) {
    let rows = current.iter().map(|row| {
        Row::new(vec![
            Cell::from(provider_label(row.provider)),
            Cell::from(row.active_email.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(row.summary.clone()),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Percentage(45),
            Constraint::Percentage(40),
        ],
    )
    .header(Row::new(["Provider", "Active account", "Summary"]).style(header_style()))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Current [{}]", provider_filter_label(filter))),
    );
    frame.render_widget(table, area);
}

fn render_accounts(frame: &mut Frame<'_>, area: Rect, accounts: &[AccountRow], selected: usize) {
    let rows = if accounts.is_empty() {
        vec![Row::new(vec![
            Cell::from("-"),
            Cell::from("No managed accounts"),
            Cell::from("-"),
            Cell::from("-"),
        ])]
    } else {
        accounts
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let state = if row.needs_relogin {
                    "relogin"
                } else if row.is_active {
                    "active"
                } else {
                    ""
                };
                let mut table_row = Row::new(vec![
                    Cell::from(provider_label(row.provider)),
                    Cell::from(row.email.clone()),
                    Cell::from(state),
                    Cell::from(row.summary.clone()),
                ]);
                if index == selected {
                    table_row = table_row.style(
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                table_row
            })
            .collect()
    };
    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Percentage(42),
            Constraint::Length(10),
            Constraint::Percentage(36),
        ],
    )
    .header(Row::new(["Provider", "Account", "State", "Summary"]).style(header_style()))
    .block(Block::default().borders(Borders::ALL).title("Accounts"));
    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, status: &str) {
    let footer = Paragraph::new(vec![
        Line::from(status.to_string()),
        Line::from("up/down/j/k select | Enter switch | r refresh | 1/2/3/a filter | q/Esc quit"),
    ])
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(footer, area);
}

fn header_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

fn provider_filter_label(provider: Option<Provider>) -> &'static str {
    match provider {
        Some(Provider::Codex) => "codex",
        Some(Provider::Claude) => "claude",
        Some(Provider::Gemini) => "gemini",
        None => "all",
    }
}

fn provider_label(provider: Provider) -> &'static str {
    match provider {
        Provider::Codex => "codex",
        Provider::Claude => "claude",
        Provider::Gemini => "gemini",
    }
}

fn provider_title(provider: Provider) -> &'static str {
    match provider {
        Provider::Codex => "Codex",
        Provider::Claude => "Claude",
        Provider::Gemini => "Gemini",
    }
}
