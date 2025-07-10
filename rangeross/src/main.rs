use color_eyre::eyre::Result;
use crossterm::event::{self, Event};
use opendal::{BlockingOperator, Builder, Operator, services};
use ratatui::{DefaultTerminal, Frame};
use std::env;

#[derive(Debug)]
struct App {
    op: Operator,
    exit: bool,
}

impl App {
    pub fn try_new() -> Result<Self> {
        let builder = services::S3::default()
            .bucket(env::var("OPENDAL_OSS_BUCKET")?.as_str())
            .endpoint(env::var("OPENDAL_OSS_ENDPOINT")?.as_str())
            .access_key_id(env::var("OPENDAL_OSS_ACCESS_KEY_ID")?.as_str())
            .secret_access_key(env::var("OPENDAL_OSS_ACCESS_KEY_SECRET")?.as_str())
            .region(
                env::var("OPENDAL_OSS_REGION")
                    .unwrap_or(String::from("auto"))
                    .as_str(),
            );

        let op = Operator::new(builder).unwrap().finish();
        Ok(Self { op, exit: false })
    }
    fn run(&mut self) {}
    fn draw(&mut self) {}
    fn handle_events(&mut self) {}
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let term = ratatui::init();
    let res = run(term);
    ratatui::restore();
    res
}

fn run(mut term: DefaultTerminal) -> Result<()> {
    loop {
        term.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}

#[cfg(test)]
mod tests {
    use crate::App;

    #[tokio::test]
    async fn list_test() {
        let app = App::try_new().unwrap();
        let arr = app.op.list_with("/").recursive(true).await.unwrap();
        for e in arr {
            println!("{}", e.path())
        }
    }
}
