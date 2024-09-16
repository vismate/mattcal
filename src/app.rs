use std::{io, time::Duration};

use chrono::{Datelike, Days, Local, NaiveDate, NaiveWeek, Weekday};
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::Constraint,
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame, Terminal,
};

enum ControlFlow {
    Continue,
    Render,
    Quit,
}

pub struct App {
    date: NaiveDate,
}

impl App {
    pub fn new() -> Self {
        Self {
            date: Local::now().date_naive(),
        }
    }

    pub fn start<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|frame| self.render(frame))?;
        loop {
            match self.handle_events()? {
                ControlFlow::Quit => break,
                ControlFlow::Render => {
                    terminal.draw(|frame| self.render(frame))?;
                }

                ControlFlow::Continue => {}
            }
        }

        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<ControlFlow> {
        if event::poll(Duration::from_secs(60))? {
            match event::read()? {
                Event::Key(key) if key.kind == event::KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => Ok(ControlFlow::Quit),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        Ok(ControlFlow::Quit)
                    }
                    KeyCode::Right => {
                        self.date = self.date.succ_opt().unwrap();
                        Ok(ControlFlow::Render)
                    }

                    KeyCode::Left => {
                        self.date = self.date.pred_opt().unwrap();
                        Ok(ControlFlow::Render)
                    }

                    KeyCode::Up => {
                        self.date = self.date.checked_sub_days(Days::new(7)).unwrap();
                        Ok(ControlFlow::Render)
                    }

                    KeyCode::Down => {
                        self.date = self.date.checked_add_days(Days::new(7)).unwrap();
                        Ok(ControlFlow::Render)
                    }

                    _ => Ok(ControlFlow::Continue),
                },
                Event::Resize(_, _) => Ok(ControlFlow::Render),
                _ => Ok(ControlFlow::Continue),
            }
        } else {
            // Some time has passed since last update so let's trigger a render
            // to potentionally chnage current date.
            Ok(ControlFlow::Render)
        }
    }

    fn render(&self, frame: &mut Frame) {
        let today = Local::now().date_naive();

        let create_cell = |day: NaiveDate| -> Cell {
            let base = Cell::from(day.day().to_string());

            let is_this_month = self.date.month() == day.month();
            let is_weekend = matches!(day.weekday(), Weekday::Sat | Weekday::Sun);
            let base = match (is_this_month, is_weekend) {
                (true, true) => base.light_red().bold(),
                (true, false) => base.light_yellow(),
                (false, true) => base.light_red().dim().italic(),
                (false, false) => base.italic(),
            };

            let is_today = day == today;
            let is_selected = day == self.date;

            match (is_today, is_selected) {
                (true, true) => base.on_green().black().bold(),
                (true, false) => base.on_light_blue().black().bold(),
                (false, true) => base.reversed().bold(),
                (false, false) => base,
            }
        };

        let row_height = u16::max(1, (frame.area().height - 4) / 6);

        let create_row = |d: NaiveWeek| -> Row {
            (0..7)
                .map(|i| create_cell(d.first_day().checked_add_days(Days::new(i)).unwrap()))
                .collect::<Row>()
                .height(row_height)
        };

        let first_day = self.date.with_day(1).unwrap();

        let rows = first_day
            .iter_weeks()
            .map(|d| d.week(Weekday::Mon))
            .take_while(|w| {
                w.first_day().month() == self.date.month()
                    || w.last_day().month() == self.date.month()
            })
            .map(create_row);

        let week_days = Row::new(
            if frame.area().width >= 80 {
                [
                    "Monday",
                    "Tuesday",
                    "Wednesday",
                    "Thursday",
                    "Friday",
                    "Saturday",
                    "Sunday",
                ]
            } else {
                ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
            }
            .map(Cell::from)
            .to_vec(),
        )
        .bottom_margin(1)
        .style(Style::new().bold());

        let title = format!("{:02}/{}", self.date.month(), self.date.year());

        let column_spacing = u16::from(frame.area().width < 40);

        let table = Table::default()
            .block(Block::default().borders(Borders::ALL).title(title.as_str()))
            .rows(rows)
            .header(week_days)
            .widths([Constraint::Fill(1); 7])
            .column_spacing(column_spacing);

        frame.render_widget(table, frame.area());
    }
}
