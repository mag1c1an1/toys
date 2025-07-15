use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use opendal::{Entry, Operator, services};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};
use std::{borrow::Cow, env, io::stdout, path::PathBuf, vec};

mod app;

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t = String::from(""))]
    path: String,
}

#[derive(Debug)]
struct App {
    // immutable
    bucket: String,
    endpoint: String,
    // mutable
    current_path: PathBuf,
    op: Operator,
    exit: bool,
    cursor: usize,
    parent_cursor: usize,

    parent_entries: Vec<Entry>,  // grand
    current_entries: Vec<Entry>, // actually parent
}

impl App {
    const PREFIX: &'static str = "s3://";
    pub async fn try_new(start_path: &str) -> Result<Self> {
        let endpoint = env::var("AWS_ENDPOINT")?;
        let id = env::var("AWS_ACCESS_KEY_ID")?;
        let key = env::var("AWS_SECRET_ACCESS_KEY")?;
        let bucket = env::var("AWS_BUCKET")?;
        let region = env::var("AWS_REGION").unwrap_or(String::from("auto"));

        let curr = PathBuf::from(start_path);

        let builder = services::S3::default()
            .endpoint(&endpoint)
            .access_key_id(&id)
            .secret_access_key(&key)
            .bucket(&bucket)
            .region(&region);
        let op = Operator::new(builder)?.finish();

        let current_entries = op.list(curr.as_path().to_str().unwrap()).await?;

        dbg!(curr.as_path());

        let parent_entries = if let Some(p) = curr.parent() {
            let p = format!("{}/", p.to_str().unwrap());
            op.list(&p).await?
        } else {
            vec![]
        };

        Ok(Self {
            current_path: curr,
            op,
            exit: false,
            bucket: bucket + "/",
            endpoint,
            cursor: 0,
            parent_cursor: 0,
            parent_entries,
            current_entries,
        })
    }

    fn current_path(&self) -> String {
        format!(
            "{}{}{}",
            Self::PREFIX,
            self.bucket,
            self.current_path.as_path().to_str().unwrap()
        )
    }

    fn current_content(&self) -> (Vec<&str>, usize) {
        (
            self.current_entries
                .iter()
                .map(|e| {
                    // if e.name().ends_with("/") {
                    //     // &e.name()[..e.name().find("/").unwrap()]
                    // } else {
                    e.name()
                    // }
                })
                .collect::<Vec<&str>>(),
            self.cursor,
        )
    }

    fn parent_content(&self) -> (Vec<&str>, usize) {
        let contents = if self.parent_entries.is_empty() {
            vec![self.bucket.as_str()]
        } else {
            self.parent_entries
                .iter()
                .map(|e| e.name())
                .collect::<Vec<&str>>()
        };
        (contents, self.parent_cursor)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();
    let args = Args::parse();

    let mut app = App::try_new(&args.path).await?;

    enable_raw_mode()?;
    set_panic_hook();
    let mut stderr = std::io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    // run
    let mut term = ratatui::init();
    let res = run(&mut term, &mut app);

    // post
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    if let Err(e) = res {
        print!("{e:?}");
    }
    Ok(())
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore(); // ignore any errors as we are already failing
        hook(panic_info);
    }));
}

/// Restore the terminal to its original state
pub fn restore() -> std::io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
enum State {
    Running,
    Exit,
}

fn handle_key(event: KeyEvent) -> Result<State> {
    if event.code == KeyCode::Char('q') {
        // quit
        return Ok(State::Exit);
    }
    Ok(State::Running)
}

fn run(term: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        term.draw(|f| ui(f, app))?;

        let state = match event::read()? {
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Key(key_event) => handle_key(key_event)?,
            Event::Mouse(_mouse_event) => State::Running,
            Event::Paste(_) => State::Running,
            Event::Resize(_, _) => State::Running,
        };

        if matches!(state, State::Exit) {
            break Ok(());
        }
    }
}

// layout
fn ui(frame: &mut Frame, app: &App) {
    let outter_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(3),
        ])
        .split(frame.area());
    // parent view ,current view and content_view
    let inner_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(outter_chunks[1]);

    let current_line = Block::default().style(Style::default());

    // first line
    let current = Paragraph::new(Text::styled(
        app.current_path(),
        Style::default().fg(Color::Green),
    ))
    .block(current_line);
    frame.render_widget(current, outter_chunks[0]);

    // parent view
    let parent_block = Block::default()
        .style(Style::default())
        .borders(Borders::RIGHT);

    let (content, cursor) = app.parent_content();

    // build lines
    let mut lines = vec![];

    for (idx, s) in content.into_iter().enumerate() {
        let style = if idx == cursor {
            Style::new().fg(Color::Blue).on_white().italic()
        } else {
            Style::default()
        };
        lines.push(Line::styled(Cow::from(s), style));
    }
    let parent_para = Paragraph::new(Text::from(lines)).block(parent_block);
    frame.render_widget(parent_para, inner_chunks[0]);

    // current view
    let curr_block = Block::default()
        .style(Style::default())
        .borders(Borders::RIGHT);
    let (content, cursor) = app.current_content();

    // build lines
    let mut lines = vec![];

    for (idx, s) in content.into_iter().enumerate() {
        let style = if idx == cursor {
            Style::new().fg(Color::Blue).on_white().italic()
        } else {
            Style::default()
        };
        lines.push(Line::styled(Cow::from(s), style));
    }

    let curr_para = Paragraph::new(Text::from(lines)).block(curr_block);

    frame.render_widget(curr_para, inner_chunks[1]);

    // preview view
}

#[cfg(test)]
mod tests {
    use opendal::services::S3;
    use url::Url;

    use crate::App;

    #[tokio::test]
    async fn list_test() {
        //    let op = S3::default()
        //    .access_key_id("")
    }

    #[tokio::test]
    async fn app_test() {
        let app = App::try_new("").await.unwrap();
        let arr = app.op.list("").await.unwrap();
        for e in arr {
            println!("{}", e.name())
        }
    }

    #[test]
    fn url_test() {
        let url = Url::parse("s3://xxxxxxx.xxxx.xxx").unwrap();
        println!("{url}");
    }

    #[test]
    fn path_test() {
        let p = ["a"];
        let s = p.join("/");
        print!("{s}");
    }
}
