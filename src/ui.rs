use std::io;

use tui::{Frame, Terminal, backend::{Backend, TermionBackend}, layout::{Constraint, Direction, Layout}, style::{Color, Style}, text::{Span, Spans}, widgets::{Gauge, Paragraph, Wrap}};
use tui::widgets::{Block, Borders};
use termion::raw::{IntoRawMode, RawTerminal};

use crate::time::{self, MetrumDateTime};

pub struct Cli<T: Backend> {
    pub terminal: Terminal<T>,
    settings: Settings
}

struct Settings {
    year_style: Style,
    day_style: Style,
    minute_style: Style,
    tick_style: Style
}

impl Cli<TermionBackend<RawTerminal<io::Stdout>>> {
    pub fn new() -> Self {
        let terminal = Terminal::new(TermionBackend::new(io::stdout().into_raw_mode().unwrap())).unwrap();
        Cli { 
            terminal,
            settings: Settings {
                year_style: Style::default().fg(Color::Blue),
                day_style: Style::default().fg(Color::LightBlue),
                minute_style: Style::default().fg(Color::Red),
                tick_style: Style::default().fg(Color::Yellow),
            }
        }
    }
}
impl<T : Backend> Cli<T> {
    pub fn render(&mut self) {
        let settings = &self.settings;
        self.terminal.draw(|f| { Self::draw(settings, f); }).unwrap();
    }

    fn draw(settings: &Settings, f: &mut Frame<T>) {
        let now = MetrumDateTime::now();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3 + 2),
                    Constraint::Min(0)
                ].as_ref()
            )
            .split(f.size());
    
        let time_paragraph = Paragraph::new(time_text(settings, &now))
            .block(
                Block::default()
                    .title("Metrum time")
                    .borders(Borders::ALL)
            ).wrap(Wrap { trim: true });
        
        f.render_widget(time_paragraph, chunks[0]);
        let gauges_block = Block::default()
            .title("Progress")
            .borders(Borders::ALL);
        
        f.render_widget(gauges_block, chunks[1]);
    
        let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Max(1),
            ].as_ref()
        )
        .split(chunks[1]);
        
        let day_gauge = Gauge::default()
            .gauge_style(settings.day_style)
            .ratio(now.day() as f64 / time::days_in_year(now.year()) as f64);
        f.render_widget(day_gauge, inner_chunks[0]);

        let minute_gauge = Gauge::default()
            .gauge_style(settings.minute_style)
            .ratio(now.minute() as f64 / time::MINUTES_PER_DAY as f64);
        f.render_widget(minute_gauge, inner_chunks[1]);

        let tick_gauge = Gauge::default()
            .gauge_style(settings.tick_style)
            .percent(now.tick() as u16);
        f.render_widget(tick_gauge, inner_chunks[2]);

    
    }
}

fn time_text<'a>(settings: &Settings, time: &MetrumDateTime) -> Spans<'a> {
    Spans::from(vec![
        Span::styled(time.year().to_string(), settings.year_style),
        Span::from("'"),
        Span::styled(format!("{:03}", time.day()), settings.day_style),
        Span::from(" "),
        Span::styled(format!("{:03}", time.minute()), settings.minute_style),
        Span::from(":"),
        Span::styled(format!("{:02}", time.tick()), settings.tick_style),
        
    ])
}