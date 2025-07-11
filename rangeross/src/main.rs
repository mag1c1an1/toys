use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use opendal::{Operator, services};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Paragraph},
};
use std::env;
use url::Url;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    path: Option<String>,
}

#[derive(Debug)]
struct App {
    // immutable
    bucket: String,
    endpoint: String,
    // mutable
    current_path: Url,
    op: Operator,
    exit: bool,
}

impl App {
    const PREFIX: &'static str = "s3://";
    pub fn try_new(start_path: &str) -> Result<Self> {
        let endpoint = env::var("AWS_ENDPOINT")?;
        let id = env::var("AWS_ACCESS_KEY_ID")?;
        let key = env::var("AWS_SECRET_ACCESS_KEY")?;
        let bucket = env::var("AWS_BUCKET")?;
        let region = env::var("AWS_REGION").unwrap_or(String::from("auto"));
        let builder = services::S3::default()
            .endpoint(&endpoint)
            .access_key_id(&id)
            .secret_access_key(&key)
            .bucket(&bucket)
            .region(&region);

        let op = Operator::new(builder).unwrap().finish();
        Ok(Self {
            current_path: Url::parse(&format!("{}{}", App::PREFIX, bucket))?,
            op,
            exit: false,
            bucket,
            endpoint,
        })
    }

    fn current_path(&self) -> &str {
        self.current_path.as_str()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let start_path = if args.path.is_some() {
        &args.path.unwrap()
    } else {
        "/"
    };

    let mut app = App::try_new(start_path)?;

    enable_raw_mode()?;
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

fn run(term: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        term.draw(|f| ui(f, app))?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let outter_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(3),
        ])
        .split(frame.area());
    let inner_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1)])
        .split(outter_chunks[1]);

    let current_line = Block::default().style(Style::default());

    let current = Paragraph::new(Text::styled(
        app.current_path(),
        Style::default().fg(Color::Green),
    ))
    .block(current_line);

    frame.render_widget(current, outter_chunks[0]);

    frame.render_widget("????", inner_chunks[0]);
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::App;

    #[tokio::test]
    async fn list_test() {
        let app = App::try_new("/").unwrap();
        let arr = app.op.list("jiax/").await.unwrap();
        for e in arr {
            println!("path: {}", e.path());
            println!("name: {}", e.name());
        }
    }

    #[test]
    fn url_test() {
        let url = Url::parse("s3://xxxxxxx.xxxx.xxx").unwrap();
        println!("{url}");
    }
}
