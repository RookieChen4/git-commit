use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, cell::RefCell, process::Command};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Type,
    Select,
}

#[derive(Debug)]
struct StatefulList<'a, T> {
    state: ListState,
    items: & 'a Vec<T>,
}

impl<'a, T> StatefulList<'a, T> {
    // 初始化
    fn with_items(items:& Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    // 向下选择
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // 向上选择
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // 取消选择
    fn unselect(&mut self) {
        self.state.select(None);
    }
}


/// App holds the state of the application
struct App<'a> {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
    state_ful_list: StatefulList<'a, & 'a str>,
}

impl <'a> App <'a> {
    fn new(items: & 'a Vec<&str>) -> App<'a> {
        App {
            input: String::new(),
            input_mode: InputMode::Type,
            messages: Vec::new(),
            state_ful_list: StatefulList::with_items(items),
        }
    }
}

impl App <'_> {
    fn set_mode(& mut self, mode: InputMode) {
        self.input_mode = mode;
    }
}

const command: [&str;3] = ["请输入任务ID", "请选择变更类型", "请输入变更范围"];

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let items = vec!["feat:     A new feature", "fix:      A bug fix", "test:     Adding missing tests"];
    let app = App::new(&items);
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    let output = Command::new("git").args(["log", "-n2"]).output().expect("failed to execute process");

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        let text = command[app.messages.len()];
        if text == "请选择变更类型" { app.set_mode(InputMode::Select) } else { app.set_mode(InputMode::Type) }
        terminal.draw(|f| ui(f, & mut app))?;
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Type => match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        app.messages.push(app.input.drain(..).collect());
                        if app.messages.len() >= command.len() { return Ok(()) }
                    },
                    _ => {}
                },
                InputMode::Select => match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Left => app.state_ful_list.unselect(),
                    KeyCode::Down => app.state_ful_list.next(),
                    KeyCode::Up => app.state_ful_list.previous(),
                    KeyCode::Enter => {
                        let index = app.state_ful_list.state.selected().unwrap_or_else(|| usize::MAX);
                        if index != usize::MAX { app.messages.push(app.state_ful_list.items[index].into()); }
                    },
                    _ => {}
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: & mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(f.size());

        render_left_area(f, chunks[0], app);

        render_right_area(f, chunks[1], app);
}

fn render_left_area<B: Backend>(f: &mut Frame<B>, chunk:tui::layout::Rect, app: & mut App) {
    match app.input_mode {
        InputMode::Select => render_select(f, chunk, app),
        _ => render_input(f, chunk, app)
    }
}


fn render_input<B: Backend>(f: &mut Frame<B>, chunk:tui::layout::Rect, app: &App) {
    let text = command[app.messages.len()];
    let chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(chunk);

        // 渲染提示
        let text = Text::from(text);
        let help_message = Paragraph::new(text);
        f.render_widget(help_message, chunk[0]);

        // 渲染input框
        let input = Paragraph::new(app.input.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Input"));

        f.render_widget(input, chunk[1]);

        // 设置input光标
        f.set_cursor(
            // Put cursor past the end of the input text
            chunk[1].x + app.input.width() as u16 + 1,
            // Move one line down, from the border to the input line
            chunk[1].y + 1,
        );

        // 展示输入信息
        let messages: Vec<ListItem> = app.messages.iter().enumerate().map(|(i , m)| {
            let content = Spans::from(Span::raw(format!("{}: {}", i, m)));
            ListItem::new(content)
        }).collect();

        let messages = List::new(messages).block(Block::default().title("Messages").borders(Borders::ALL));

        f.render_widget(messages, chunk[2]);
}

fn render_select<B: Backend>(f: &mut Frame<B>, chunk:tui::layout::Rect, app: & mut App) {
    let items: Vec<ListItem> = app.state_ful_list.items.iter().map(|i| {
        let content = Spans::from(Span::raw(format!("{}", i)));
        ListItem::new(content).style(Style::default().fg(Color::Black).bg(Color::White))
    }).collect();

    let items = List::new(items).block(Block::default().title("Selected").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::LightGreen).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, chunk,  & mut app.state_ful_list.state);
}

fn render_right_area<B: Backend>(f: &mut Frame<B>, chunk:tui::layout::Rect, app: &App) {

    let block = Block::default().title("git log").borders(Borders::ALL);

    f.render_widget(block, chunk);

    let chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(1)].as_ref())
        .split(chunk);

    let output = Command::new("git").args(["log", "-n2"]).output().expect("failed to execute process");

    let stdout  = String::from_utf8(output.stdout);
    let stdout  = stdout.unwrap_or_default();
    let stdout  = stdout.split('\n').collect::<Vec<&str>>();
    let stdout: Vec<ListItem> = stdout.iter().map(|i| {
        let content = Spans::from(Span::raw(format!("  {}", i)));
        ListItem::new(content).style(Style::default())
    }).collect();

    let stdout = List::new(stdout);

    f.render_widget(stdout, chunk[0]);
}