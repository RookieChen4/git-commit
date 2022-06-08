use std::io;
use std::fs::File;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    time::{Duration, Instant},
    process::Command
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

#[derive(Debug)]
struct StatefulList<'a, T> {
    state: ListState,
    items: & 'a Vec<T>,
}

impl<'a, T> StatefulList<'a, T> {
    fn with_items(items:& Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

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

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[derive(Debug)]
struct App<'a> {
    items: StatefulList<'a, (&'a str, &'a str, usize)>,
}

impl<'a> App <'a> {
    fn new<'b>(items: & 'b Vec<(&str,&str, usize)>) -> App<'b> {
			App {
				items: StatefulList::with_items(items)
			}
		}
}

fn main() {
    // 存放输入
    let mut command_line: Vec<String> = vec![];
    // read the json file
    let f = File::open("custom.json").unwrap();
    let v: serde_json::Value = serde_json::from_reader(f).unwrap();
    
    // 将json转成数组
    let change_type = &v["ChangeType"];
    let mut array: Vec<(&str, &str, usize)> = vec![];
    let mut index = 0;
    change_type.as_array().unwrap().iter().for_each(|o| {
        let mut temp: (&str, &str, usize) = ("", "", 0);
        o.as_object().unwrap().iter().for_each(|(key, value)|{
            if key == "name" {
                temp.0 = value.as_str().unwrap();
                temp.2 = index;
                index += 1;
            } else {
                temp.1 = value.as_str().unwrap();
            }
        });
        let _ = &array.push(temp);
    });

    let commit_map = &v["messages"];

    // 将json转成数组
    let mut command_map = vec![];
    commit_map.as_array().unwrap().iter().for_each(|message_object| {
        let mut type_key = "";
        let mut placeholder = "";
        message_object.as_object().unwrap().iter().for_each(|(key, value)| {
            if key == "type" {
                type_key = value.as_str().unwrap();
            }
            if key == "placeholder" {
                placeholder = value.as_str().unwrap();
            }
        });
        command_map.push((type_key, placeholder));
    });

    // 执行命令
    for (key, value) in command_map {
        println!("{}: {}", key, value);
        let mut input = String::new();
        if key == "ChangeType" {
            input = open_terminal(&array).unwrap().to_string();
            input = array[input.parse::<usize>().unwrap()].1.to_string();
        } else {
            io::stdin().read_line(&mut input).unwrap();
        }
        command_line.push(input.trim().to_string());
    }

    println!("{:?}", command_line);
    let command = command_line.join(" ");
    Command::new("git")
        .args(["commit", "-m", &command])
        .output()
        .expect("failed to execute process");
}

fn open_terminal(array: & Vec<(&str, &str, usize)>) -> Result<usize, Box<dyn Error>> {
    // enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
     // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new(array);
    let res = run_app(&mut terminal, app, tick_rate);
    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = &res {
        println!("{:?}", err)
    }

    // Ok(array[res.unwrap()].0)
    Ok(res.unwrap())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<usize> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Enter => return Ok(app.items.state.selected().unwrap_or_else(|| 0)),
                    KeyCode::Left => app.items.unselect(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let lines = vec![Spans::from(i.0)];
            // for _ in 0..i.2 {
            //     lines.push(Spans::from(Span::styled(
            //         "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            //         Style::default().add_modifier(Modifier::ITALIC),
            //     )));
            // }
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

    let block = Block::default()
        .title(app.items.state.selected().unwrap_or_else(|| 0).to_string())
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}