use std::io::{self, Stdout};
use std::time::Duration;

use aah_core::cli_facade::{
    AccountQuotaRow, AccountRow, CliFacade, CurrentRow, Provider, SwitchSelection,
};
use aah_core::time_utils::format_refresh_countdown;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend, TestBackend};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
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
    source_accounts: Vec<AccountRow>,
    accounts: Vec<AccountRow>,
    current: Vec<CurrentRow>,
    filter: Option<Provider>,
    selected: usize,
    status: String,
    mode: TuiMode,
    search_query: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TuiMode {
    Normal,
    Search,
    LabelInput {
        provider: Provider,
        id: String,
        email: String,
        input: String,
    },
    ConfirmDelete {
        provider: Provider,
        id: String,
        email: String,
    },
    Detail,
    Help,
}

impl TuiModel {
    fn from_facade(facade: &CliFacade) -> Result<Self, String> {
        let mut model = Self {
            source_accounts: Vec::new(),
            accounts: Vec::new(),
            current: Vec::new(),
            filter: None,
            selected: 0,
            status: "Ready".to_string(),
            mode: TuiMode::Normal,
            search_query: String::new(),
        };
        model.reload(facade)?;
        Ok(model)
    }

    fn reload(&mut self, facade: &CliFacade) -> Result<(), String> {
        self.source_accounts = facade
            .list(self.filter)
            .map_err(|error| error.to_string())?;
        self.current = facade
            .current(self.filter)
            .map_err(|error| error.to_string())?;
        self.apply_search_filter();
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
        match self.mode.clone() {
            TuiMode::Normal => self.handle_normal_key(facade, key),
            TuiMode::Search => self.handle_search_key(key),
            TuiMode::LabelInput { .. } => self.handle_label_key(facade, key),
            TuiMode::ConfirmDelete { .. } => self.handle_delete_key(facade, key),
            TuiMode::Detail | TuiMode::Help => self.handle_panel_key(key),
        }
    }

    fn handle_normal_key(&mut self, facade: &CliFacade, key: KeyEvent) -> Result<bool, String> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(true),
            KeyCode::Char('?') => {
                self.mode = TuiMode::Help;
                self.status = "Help open".to_string();
                Ok(false)
            }
            KeyCode::Char('/') => {
                self.enter_search();
                Ok(false)
            }
            KeyCode::Char('i') => {
                if self.selected_account().is_some() {
                    self.mode = TuiMode::Detail;
                    self.status = "Account details open".to_string();
                } else {
                    self.status = "No account selected".to_string();
                }
                Ok(false)
            }
            KeyCode::Char('l') => {
                self.begin_label_input();
                Ok(false)
            }
            KeyCode::Char('d') => {
                self.begin_delete_confirmation();
                Ok(false)
            }
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

    fn handle_search_key(&mut self, key: KeyEvent) -> Result<bool, String> {
        match key.code {
            KeyCode::Esc => {
                self.search_query.clear();
                self.apply_search_filter();
                self.mode = TuiMode::Normal;
                self.status = "Search cleared".to_string();
            }
            KeyCode::Enter => {
                self.mode = TuiMode::Normal;
                self.status = search_status(&self.search_query, self.accounts.len());
            }
            KeyCode::Backspace => {
                let mut query = self.search_query.clone();
                query.pop();
                self.apply_search_query(&query);
            }
            KeyCode::Char(value) => {
                let mut query = self.search_query.clone();
                query.push(value);
                self.apply_search_query(&query);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_label_key(&mut self, facade: &CliFacade, key: KeyEvent) -> Result<bool, String> {
        match key.code {
            KeyCode::Esc => {
                self.mode = TuiMode::Normal;
                self.status = "Label edit cancelled".to_string();
            }
            KeyCode::Enter => {
                if let Err(error) = self.confirm_label(facade) {
                    self.mode = TuiMode::Normal;
                    self.status = format!("Label failed: {error}");
                }
            }
            KeyCode::Backspace => {
                if let TuiMode::LabelInput { input, .. } = &mut self.mode {
                    input.pop();
                }
            }
            KeyCode::Char(value) => {
                if let TuiMode::LabelInput { input, .. } = &mut self.mode {
                    input.push(value);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_delete_key(&mut self, facade: &CliFacade, key: KeyEvent) -> Result<bool, String> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Err(error) = self.confirm_delete(facade) {
                    self.mode = TuiMode::Normal;
                    self.status = format!("Delete failed: {error}");
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = TuiMode::Normal;
                self.status = "Delete cancelled".to_string();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_panel_key(&mut self, key: KeyEvent) -> Result<bool, String> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = TuiMode::Normal;
                self.status = "Ready".to_string();
            }
            KeyCode::Char('?') => {
                self.mode = match self.mode {
                    TuiMode::Help => TuiMode::Normal,
                    _ => TuiMode::Help,
                };
            }
            KeyCode::Char('i') => {
                self.mode = match self.mode {
                    TuiMode::Detail => TuiMode::Normal,
                    _ => TuiMode::Detail,
                };
            }
            _ => {}
        }
        Ok(false)
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

    fn enter_search(&mut self) {
        self.mode = TuiMode::Search;
        self.status = search_status(&self.search_query, self.accounts.len());
    }

    fn apply_search_query(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.apply_search_filter();
        self.status = search_status(&self.search_query, self.accounts.len());
    }

    fn apply_search_filter(&mut self) {
        let query = self.search_query.trim();
        self.accounts = if query.is_empty() {
            self.source_accounts.clone()
        } else {
            self.source_accounts
                .iter()
                .filter(|account| account_matches(account, query))
                .cloned()
                .collect()
        };
        self.clamp_selection();
    }

    fn selected_account(&self) -> Option<&AccountRow> {
        self.accounts.get(self.selected)
    }

    fn begin_label_input(&mut self) {
        let Some(account) = self.selected_account() else {
            self.status = "No account selected".to_string();
            return;
        };
        self.mode = TuiMode::LabelInput {
            provider: account.provider,
            id: account.id.clone(),
            email: account.email.clone(),
            input: account.label.clone().unwrap_or_default(),
        };
        self.status = "Editing label; Enter saves, empty clears".to_string();
    }

    #[cfg(test)]
    fn replace_label_input(&mut self, label: &str) {
        if let TuiMode::LabelInput { input, .. } = &mut self.mode {
            *input = label.to_string();
        }
    }

    fn confirm_label(&mut self, facade: &CliFacade) -> Result<(), String> {
        let TuiMode::LabelInput {
            provider,
            id,
            email,
            input,
        } = self.mode.clone()
        else {
            return Ok(());
        };
        let label = if input.trim().is_empty() {
            None
        } else {
            Some(input)
        };
        let outcome = facade
            .label(provider, SwitchSelection::Id(id), label)
            .map_err(|error| error.to_string())?;
        self.mode = TuiMode::Normal;
        self.status = match outcome.label {
            Some(label) => format!(
                "Labelled {} account {} as \"{}\"",
                provider_title(provider),
                email,
                label
            ),
            None => format!(
                "Cleared label for {} account {}",
                provider_title(provider),
                email
            ),
        };
        self.reload(facade)
    }

    fn begin_delete_confirmation(&mut self) {
        let Some(account) = self.selected_account() else {
            self.status = "No account selected".to_string();
            return;
        };
        let provider = account.provider;
        let id = account.id.clone();
        let email = account.email.clone();
        self.mode = TuiMode::ConfirmDelete {
            provider,
            id,
            email: email.clone(),
        };
        self.status = format!(
            "Delete {} account {}? Press y to confirm, n/Esc to cancel",
            provider_title(provider),
            email
        );
    }

    fn confirm_delete(&mut self, facade: &CliFacade) -> Result<(), String> {
        let TuiMode::ConfirmDelete {
            provider,
            id,
            email,
        } = self.mode.clone()
        else {
            return Ok(());
        };
        facade
            .remove(provider, SwitchSelection::Id(id))
            .map_err(|error| error.to_string())?;
        self.mode = TuiMode::Normal;
        self.status = format!("Deleted {} account {}", provider_title(provider), email);
        self.reload(facade)
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
        self.search_query.clear();
        self.mode = TuiMode::Normal;
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
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(9),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_header(frame, chunks[0], model);
    render_provider_tabs(frame, chunks[1], model.filter);
    render_current(frame, chunks[2], &model.current, model.filter);
    render_main_panel(frame, chunks[3], model);
    render_footer(frame, chunks[4], model);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, model: &TuiModel) {
    let search_suffix = if model.search_query.trim().is_empty() {
        String::new()
    } else {
        format!(" | search: {}", model.search_query)
    };
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "AI Accounts Hub",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!(
            "Terminal account switchboard | filter: {} | accounts: {}/{}{}",
            provider_filter_label(model.filter),
            model.accounts.len(),
            model.source_accounts.len(),
            search_suffix
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title("aah tui"));
    frame.render_widget(header, area);
}

fn render_provider_tabs(frame: &mut Frame<'_>, area: Rect, filter: Option<Provider>) {
    let tabs = Paragraph::new(Line::from(vec![
        provider_tab_span("1 Codex", filter == Some(Provider::Codex)),
        Span::raw("  "),
        provider_tab_span("2 Claude", filter == Some(Provider::Claude)),
        Span::raw("  "),
        provider_tab_span("3 Gemini", filter == Some(Provider::Gemini)),
        Span::raw("    "),
        Span::styled(
            "a All",
            if filter.is_none() {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Gray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Provider tabs"),
    );
    frame.render_widget(tabs, area);
}

fn provider_tab_span(label: &'static str, active: bool) -> Span<'static> {
    if active {
        Span::styled(
            format!(" {label} "),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(format!(" {label} "), Style::default().fg(Color::Cyan))
    }
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
            Cell::from(match row.active_email.as_deref() {
                Some(email) => account_display(row.active_label.as_deref(), email),
                None => "-".to_string(),
            }),
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

fn render_main_panel(frame: &mut Frame<'_>, area: Rect, model: &TuiModel) {
    match model.mode {
        TuiMode::Help => render_help(frame, area),
        TuiMode::Detail => render_detail(frame, area, model.selected_account()),
        _ => render_accounts(frame, area, &model.accounts, model.selected),
    }
}

fn render_help(frame: &mut Frame<'_>, area: Rect) {
    let help = Paragraph::new(vec![
        Line::from("up/down/j/k select account"),
        Line::from("Enter switch selected account"),
        Line::from("/ search accounts by provider, email, label, or id"),
        Line::from("l label selected account; empty label clears it"),
        Line::from("d delete selected account with confirmation"),
        Line::from("i details for selected account"),
        Line::from("1 Codex | 2 Claude | 3 Gemini | a All"),
        Line::from("r refresh quota | q/Esc quit or close panel"),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Keyboard help"),
    );
    frame.render_widget(help, area);
}

fn render_detail(frame: &mut Frame<'_>, area: Rect, account: Option<&AccountRow>) {
    let lines = match account {
        Some(account) => {
            let mut lines = vec![
                Line::from(format!("provider: {}", provider_label(account.provider))),
                Line::from(format!("id: {}", account.id)),
                Line::from(format!("email: {}", account.email)),
                Line::from(format!(
                    "label: {}",
                    account.label.as_deref().unwrap_or("-")
                )),
                Line::from(format!(
                    "active: {}",
                    if account.is_active { "yes" } else { "no" }
                )),
                Line::from(format!(
                    "relogin: {}",
                    if account.needs_relogin { "yes" } else { "no" }
                )),
                Line::from(format!("summary: {}", account.summary)),
            ];
            if !account.quota_rows.is_empty() || account.quota_meta.is_some() {
                lines.push(Line::from("quotas:"));
                for quota in &account.quota_rows {
                    lines.push(Line::from(format!("  {}", quota_display_plain(quota))));
                }
                if let Some(meta) = &account.quota_meta {
                    lines.push(Line::from(format!("  {meta}")));
                }
            }
            lines
        }
        None => vec![Line::from("No account selected")],
    };
    let detail = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Account details"),
    );
    frame.render_widget(detail, area);
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
                    Cell::from(account_display(row.label.as_deref(), &row.email)),
                    Cell::from(state),
                    Cell::from(quota_cell(row)),
                ])
                .height(account_row_height(row));
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
            Constraint::Percentage(34),
            Constraint::Length(10),
            Constraint::Percentage(44),
        ],
    )
    .header(Row::new(["Provider", "Account", "State", "Quota"]).style(header_style()))
    .block(Block::default().borders(Borders::ALL).title("Accounts"));
    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, model: &TuiModel) {
    let controls = match &model.mode {
        TuiMode::Normal => {
            "j/k | Enter switch | / search | i detail | l label | d delete | r refresh | ? help | q/Esc quit".to_string()
        }
        TuiMode::Search => format!(
            "search: {} | type to filter | Backspace edit | Enter keep | Esc clear",
            model.search_query
        ),
        TuiMode::LabelInput { input, .. } => format!(
            "label: {} | Enter save | empty clears label | Backspace edit | Esc cancel",
            input
        ),
        TuiMode::ConfirmDelete { email, .. } => {
            format!("delete {email}? | y confirm | n/Esc cancel")
        }
        TuiMode::Detail => "Account details | Esc/q close | i toggle details".to_string(),
        TuiMode::Help => "Keyboard help | Esc/q close | ? toggle help".to_string(),
    };
    let footer = Paragraph::new(vec![Line::from(model.status.clone()), Line::from(controls)])
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(footer, area);
}

fn header_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

fn search_status(query: &str, count: usize) -> String {
    if query.trim().is_empty() {
        format!("Search: showing {count} account(s)")
    } else {
        format!("Search \"{}\": {count} account(s)", query.trim())
    }
}

fn account_matches(account: &AccountRow, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    provider_label(account.provider).contains(&query)
        || account.id.to_ascii_lowercase().contains(&query)
        || account.email.to_ascii_lowercase().contains(&query)
        || account
            .label
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&query)
        || account.summary.to_ascii_lowercase().contains(&query)
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

fn account_display(label: Option<&str>, email: &str) -> String {
    match label {
        Some(label) => format!("{label} <{email}>"),
        None => email.to_string(),
    }
}

fn quota_cell(account: &AccountRow) -> Text<'static> {
    let mut lines = if account.quota_rows.is_empty() {
        vec![Line::from(account.summary.clone())]
    } else {
        account
            .quota_rows
            .iter()
            .map(quota_line)
            .collect::<Vec<_>>()
    };
    if let Some(meta) = &account.quota_meta {
        lines.push(Line::from(vec![Span::styled(
            format!("• {meta}"),
            Style::default().fg(Color::DarkGray),
        )]));
    }
    Text::from(lines)
}

fn quota_line(quota: &AccountQuotaRow) -> Line<'static> {
    let percent = quota.remaining_percent.map(clamp_percent);
    let tone = percent.map(quota_tone).unwrap_or(Color::DarkGray);
    let (filled, empty) = progress_bar_segments(percent.unwrap_or(0));
    let percent_label = match percent {
        Some(percent) => format!("{percent:>3}%"),
        None => "sync".to_string(),
    };
    let refresh_label = quota
        .remaining_percent
        .map(|_| format_refresh_countdown(quota.refresh_at.as_deref()))
        .unwrap_or_else(|| "--:--".to_string());

    Line::from(vec![
        Span::styled(
            format!("{:<10}", quota.label),
            Style::default().fg(Color::Gray),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{percent_label:>4}"),
            Style::default().fg(tone).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(filled, Style::default().fg(tone)),
        Span::styled(empty, Style::default().fg(Color::DarkGray)),
        Span::styled("  ↻ ", Style::default().fg(Color::DarkGray)),
        Span::styled(refresh_label, Style::default().fg(Color::Gray)),
    ])
}

fn quota_display_plain(quota: &AccountQuotaRow) -> String {
    let percent = quota.remaining_percent.map(clamp_percent);
    let (filled, empty) = progress_bar_segments(percent.unwrap_or(0));
    let percent_label = percent
        .map(|percent| format!("{percent:>3}%"))
        .unwrap_or_else(|| "sync".to_string());
    let refresh_label = quota
        .remaining_percent
        .map(|_| format_refresh_countdown(quota.refresh_at.as_deref()))
        .unwrap_or_else(|| "--:--".to_string());

    format!(
        "{:<10} {:>4}  {}{}  ↻ {}",
        quota.label, percent_label, filled, empty, refresh_label
    )
}

fn progress_bar_segments(percent: u8) -> (String, String) {
    const WIDTH: u8 = 10;
    let mut filled = ((percent as u16 * WIDTH as u16) + 50) / 100;
    if percent > 0 {
        filled = filled.max(1);
    }
    let filled = filled.min(WIDTH as u16) as usize;
    let empty = WIDTH as usize - filled;
    ("▰".repeat(filled), "▱".repeat(empty))
}

fn clamp_percent(percent: u8) -> u8 {
    percent.min(100)
}

fn quota_tone(percent: u8) -> Color {
    if percent <= 10 {
        Color::Red
    } else if percent <= 30 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn account_row_height(account: &AccountRow) -> u16 {
    let quota_lines = if account.quota_rows.is_empty() {
        1
    } else {
        account.quota_rows.len()
    };
    let meta_lines = usize::from(account.quota_meta.is_some());
    (quota_lines + meta_lines).max(1).min(u16::MAX as usize) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use aah_core::bootstrap::BootstrapContext;
    use std::fs;
    use std::path::Path;

    #[test]
    fn search_filters_accounts_by_label_email_and_provider() {
        let mut model = model_with_accounts(vec![
            account(
                Provider::Codex,
                "codex-1",
                "codex@example.com",
                Some("Work"),
            ),
            account(Provider::Claude, "claude-1", "claude@example.com", None),
        ]);

        model.enter_search();
        model.apply_search_query("work");
        assert_eq!(model.accounts.len(), 1);
        assert_eq!(model.accounts[0].email, "codex@example.com");

        model.apply_search_query("claude");
        assert_eq!(model.accounts.len(), 1);
        assert_eq!(model.accounts[0].email, "claude@example.com");

        model.apply_search_query("missing");
        assert!(model.accounts.is_empty());
        assert_eq!(model.selected, 0);
    }

    #[test]
    fn help_panel_renders_keyboard_shortcuts() {
        let mut model = model_with_accounts(Vec::new());
        model.mode = TuiMode::Help;

        let snapshot = render_snapshot(&model).expect("snapshot");

        assert!(snapshot.contains("Keyboard help"));
        assert!(snapshot.contains("/ search"));
        assert!(snapshot.contains("l label"));
        assert!(snapshot.contains("d delete"));
        assert!(snapshot.contains("i details"));
    }

    #[test]
    fn detail_panel_renders_selected_account_metadata() {
        let mut model = model_with_accounts(vec![account(
            Provider::Codex,
            "codex-1",
            "codex@example.com",
            Some("Work"),
        )]);
        model.mode = TuiMode::Detail;

        let snapshot = render_snapshot(&model).expect("snapshot");

        assert!(snapshot.contains("Account details"));
        assert!(snapshot.contains("id: codex-1"));
        assert!(snapshot.contains("email: codex@example.com"));
        assert!(snapshot.contains("label: Work"));
    }

    #[test]
    fn account_table_renders_quota_progress_and_reset_times() {
        let model = model_with_accounts(vec![account_with_quota(
            Provider::Codex,
            "codex-1",
            "codex@example.com",
            Some("Work"),
            vec![
                quota("5h", Some(82), Some("1800000000")),
                quota("Weekly", Some(64), Some("1800600000")),
            ],
            Some("Credits 12.5"),
        )]);

        let snapshot = render_snapshot(&model).expect("snapshot");

        assert!(snapshot.contains("5h"));
        assert!(snapshot.contains("▰▰▰▰▰▰▰▰▱▱"));
        assert!(snapshot.contains("82%"));
        assert!(snapshot.contains("Weekly"));
        assert!(snapshot.contains("↻"));
        assert!(!snapshot.contains("[########--]"));
        assert!(snapshot.contains("Credits 12.5"));
    }

    #[test]
    fn label_input_updates_selected_account_label() {
        let temp = tempfile::tempdir().expect("temp dir");
        write_codex_account(temp.path(), "codex-1", "codex@example.com", None);
        let facade = test_facade(temp.path());
        let mut model = TuiModel::from_facade(&facade).expect("model");

        model.begin_label_input();
        model.replace_label_input("Work");
        model.confirm_label(&facade).expect("label");

        assert_eq!(model.accounts.len(), 1);
        assert_eq!(model.accounts[0].label.as_deref(), Some("Work"));
        assert_eq!(
            model.status,
            "Labelled Codex account codex@example.com as \"Work\""
        );
    }

    #[test]
    fn delete_confirmation_removes_selected_account() {
        let temp = tempfile::tempdir().expect("temp dir");
        write_codex_account(temp.path(), "codex-1", "codex@example.com", None);
        let facade = test_facade(temp.path());
        let mut model = TuiModel::from_facade(&facade).expect("model");

        model.begin_delete_confirmation();
        model.confirm_delete(&facade).expect("delete");

        assert!(model.accounts.is_empty());
        assert_eq!(model.status, "Deleted Codex account codex@example.com");
    }

    fn model_with_accounts(accounts: Vec<AccountRow>) -> TuiModel {
        TuiModel {
            source_accounts: accounts.clone(),
            accounts,
            current: Vec::new(),
            filter: None,
            selected: 0,
            status: "Ready".to_string(),
            mode: TuiMode::Normal,
            search_query: String::new(),
        }
    }

    fn account(provider: Provider, id: &str, email: &str, label: Option<&str>) -> AccountRow {
        account_with_quota(provider, id, email, label, Vec::new(), None)
    }

    fn account_with_quota(
        provider: Provider,
        id: &str,
        email: &str,
        label: Option<&str>,
        quota_rows: Vec<aah_core::cli_facade::AccountQuotaRow>,
        quota_meta: Option<&str>,
    ) -> AccountRow {
        AccountRow {
            provider,
            id: id.to_string(),
            email: email.to_string(),
            label: label.map(ToString::to_string),
            is_active: false,
            summary: "quota 80%".to_string(),
            quota_rows,
            quota_meta: quota_meta.map(ToString::to_string),
            needs_relogin: false,
        }
    }

    fn quota(
        label: &str,
        remaining_percent: Option<u8>,
        refresh_at: Option<&str>,
    ) -> aah_core::cli_facade::AccountQuotaRow {
        aah_core::cli_facade::AccountQuotaRow {
            label: label.to_string(),
            remaining_percent,
            refresh_at: refresh_at.map(ToString::to_string),
        }
    }

    fn test_facade(root: &Path) -> CliFacade {
        CliFacade::new(BootstrapContext {
            managed_root: root.to_path_buf(),
            user_home: root.to_path_buf(),
            import_warnings: Vec::new(),
        })
    }

    fn write_codex_account(root: &Path, id: &str, email: &str, label: Option<&str>) {
        let codex_dir = root.join("codex");
        let managed_home = codex_dir.join("managed-codex-homes").join(id);
        fs::create_dir_all(&managed_home).expect("managed home");
        let label_field = label
            .map(|label| format!(r#","label": "{label}""#))
            .unwrap_or_default();
        let index = format!(
            r#"{{
  "version": 1,
  "accounts": [
    {{
      "id": "{id}",
      "email": "{email}",
      "account_id": "acct-{id}",
      "plan": "Plus",
      "managed_home_path": "{}",
      "created_at": "0",
      "updated_at": "0",
      "last_authenticated_at": "0"{label_field}
    }}
  ]
}}"#,
            managed_home.display()
        );
        fs::create_dir_all(&codex_dir).expect("codex dir");
        fs::write(codex_dir.join("accounts.json"), index).expect("accounts.json");
    }
}
