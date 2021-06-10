use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::io;

use log::info;
use crossterm::{input, AlternateScreen, InputEvent, KeyEvent, RawScreen};
use structopt::StructOpt;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui::layout::{Constraint, Corner, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, SelectableList, Text, Widget};

use crossterm::IntoRawMode;

use super::util::{Event, Events};
// use super::Level;
use log::Level;

pub struct App<'a> {
    items: Vec<&'a str>,
    selected: Option<usize>,
    events: Vec<(&'a str, &'a str)>,
    info_style: Style,
    warning_style: Style,
    error_style: Style,
    critical_style: Style,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            items: vec![
                "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9",
                "Item10", "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17",
                "Item18", "Item19", "Item20", "Item21", "Item22", "Item23", "Item24",
            ],
            selected: None,
            events: vec![
                ("Event1", "INFO"),
                ("Event2", "INFO"),
                ("Event3", "CRITICAL"),
                ("Event4", "ERROR"),
                ("Event5", "INFO"),
                ("Event6", "INFO"),
                ("Event7", "WARNING"),
                ("Event8", "INFO"),
                ("Event9", "INFO"),
                ("Event10", "INFO"),
                ("Event11", "CRITICAL"),
                ("Event12", "INFO"),
                ("Event13", "INFO"),
                ("Event14", "INFO"),
                ("Event15", "INFO"),
                ("Event16", "INFO"),
                ("Event17", "ERROR"),
                ("Event18", "ERROR"),
                ("Event19", "INFO"),
                ("Event20", "INFO"),
                ("Event21", "WARNING"),
                ("Event22", "INFO"),
                ("Event23", "INFO"),
                ("Event24", "WARNING"),
                ("Event25", "INFO"),
                ("Event26", "INFO"),
            ],
            info_style: Style::default().fg(Color::White),
            warning_style: Style::default().fg(Color::Yellow),
            error_style: Style::default().fg(Color::Magenta),
            critical_style: Style::default().fg(Color::Red),
        }
    }

    fn advance(&mut self) {
        let event = self.events.pop().unwrap();
        self.events.insert(0, event);
    }
}


pub fn display() -> Result<(), crossterm::ErrorKind> {
	// let alt = AlternateScreen::to_alternate(true).unwrap();
    let backend = CrosstermBackend::new();//with_alternate_screen(alt)?;
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

	let mut app = App::new();
	let events = Events::new();

	loop {
		terminal.draw(|mut f| {
			// println!("draw");

			let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            let style = Style::default().fg(Color::White).bg(Color::Black);
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("List"))
                .items(&app.items)
                .select(app.selected)
                .style(style)
                .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol(">")
                .render(&mut f, chunks[0]);

			{
				let lines = super::PRINT_LINES.lock().unwrap();

                let events = lines.iter().map(|&(ref evt, ref level)| {
                    Text::styled(
                        format!("{}: {}", level, evt),
                        match level {
                            // "CRITICAL" => app.critical_style,
                            Level::Info => app.info_style,
                            Level::Warn => app.warning_style,
                            Level::Error => app.error_style,
							_ => app.critical_style
                        },
                    )
                });

                List::new(events)
                    .block(Block::default().borders(Borders::ALL).title("Output"))
                    .start_corner(Corner::TopLeft)
                    .render(&mut f, chunks[1]);
            }
		})?;

		match events.next().expect("next events") {
            Event::Input(input) => {
				info!("{:?}", input);

				match input {
					KeyEvent::Char('q') => {
						break;
					}

					KeyEvent::Left => {
						app.selected = None;
					}

					KeyEvent::Down => {
						app.selected = if let Some(selected) = app.selected {
							if selected >= app.items.len() - 1 {
								Some(0)
							} else {
								Some(selected + 1)
							}
						} else {
							Some(0)
						}
					}

					KeyEvent::Up => {
						app.selected = if let Some(selected) = app.selected {
							if selected > 0 {
								Some(selected - 1)
							} else {
								Some(app.items.len() - 1)
							}
						} else {
							Some(0)
						}
					}

					_ => {}
				}
			}

            Event::Tick => {
                app.advance();
            }
        }
	}

    Ok(())
}