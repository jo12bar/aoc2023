use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, ComponentError, Frame};
use crate::{action::Action, config::key_event_to_string};

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Processing,
}

#[derive(Default)]
pub struct Home {
    show_help: bool,
    counter: usize,
    app_ticker: usize,
    render_ticker: usize,
    mode: Mode,
    input: Input,
    action_tx: Option<UnboundedSender<Action>>,
    text: Vec<String>,
    last_events: Vec<KeyEvent>,
}

impl Home {
    pub fn new() -> Self {
        Self::default()
    }

    fn tick(&mut self) {
        tracing::trace!(
            previous = self.app_ticker,
            next = self.app_ticker.saturating_add(1),
            "home tick"
        );
        self.app_ticker = self.app_ticker.saturating_add(1);
        self.last_events.drain(..);
    }

    fn render_tick(&mut self) {
        tracing::trace!(
            previous = self.render_ticker,
            next = self.render_ticker.saturating_add(1),
            "home render tick"
        );
        self.render_ticker = self.render_ticker.saturating_add(1);
    }

    fn add(&mut self, s: String) {
        self.text.push(s)
    }

    fn schedule_increment(&mut self, i: usize) {
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
            tx.send(Action::EnterProcessing).unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
            tx.send(Action::Increment(i)).unwrap();
            tx.send(Action::ExitProcessing).unwrap();
        });
    }

    fn schedule_decrement(&mut self, i: usize) {
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
            tx.send(Action::EnterProcessing).unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
            tx.send(Action::Decrement(i)).unwrap();
            tx.send(Action::ExitProcessing).unwrap();
        });
    }

    fn increment(&mut self, i: usize) {
        self.counter = self.counter.saturating_add(i);
    }

    fn decrement(&mut self, i: usize) {
        self.counter = self.counter.saturating_sub(i);
    }
}

impl Component for Home {
    fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> Result<(), ComponentError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, ComponentError> {
        self.last_events.push(key);

        let action = match self.mode {
            Mode::Normal | Mode::Processing => return Ok(None),

            Mode::Insert => match key.code {
                KeyCode::Esc => Action::EnterNormal,

                KeyCode::Enter => {
                    if let Some(sender) = &self.action_tx {
                        if let Err(e) =
                            sender.send(Action::CompleteInput(self.input.value().to_string()))
                        {
                            tracing::error!("Failed to send \"complete input\" action: {e:?}");
                        }
                    }
                    Action::EnterNormal
                },

                _ => {
                    self.input.handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                },
            },
        };

        Ok(Some(action))
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, ComponentError> {
        match action {
            Action::Tick => self.tick(),
            Action::Render => self.render_tick(),
            Action::ToggleShowHelp => self.show_help = !self.show_help,
            Action::ScheduleIncrement => self.schedule_increment(1),
            Action::ScheduleDecrement => self.schedule_decrement(1),
            Action::Increment(i) => self.increment(i),
            Action::Decrement(i) => self.decrement(i),
            Action::CompleteInput(s) => self.add(s),
            Action::EnterNormal => {
                self.mode = Mode::Normal;
            },
            Action::EnterInsert => {
                self.mode = Mode::Insert;
            },
            Action::EnterProcessing => {
                self.mode = Mode::Processing;
            },
            Action::ExitProcessing => {
                self.mode = Mode::Normal;
            },
            _ => {},
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<(), ComponentError> {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100), Constraint::Min(3)])
            .split(rect);

        let mut text = vec![
            Line::from(""),
            Line::from("Type into input and hit enter to display here."),
            Line::from(""),
            Line::from(format!("Render ticker: {}", self.render_ticker)),
            Line::from(format!("App ticker: {}", self.app_ticker)),
            Line::from(format!("Counter: {}", self.counter)),
            Line::from(vec![
                "Press ".into(),
                Span::styled("j", Style::default().fg(Color::Red)),
                " to increment or ".into(),
                Span::styled("k", Style::default().fg(Color::Red)),
                " to decrement.".into(),
            ]),
            Line::from(""),
        ];

        let mut user_text = self
            .text
            .iter()
            .map(|l| Line::from(l.clone()))
            .collect::<Vec<_>>();

        text.append(&mut user_text);

        f.render_widget(
            Paragraph::new(text)
                .block(
                    Block::default()
                        .title("ratatui async template")
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_style(match self.mode {
                            Mode::Processing => Style::default().fg(Color::Yellow),
                            _ => Style::default(),
                        })
                        .border_type(BorderType::Rounded),
                )
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center),
            rects[0],
        );

        let width = rects[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .style(match self.mode {
                Mode::Insert => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(vec![
                        "Enter input mode ".into(),
                        Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "/",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Gray),
                        ),
                        Span::styled(" to start, ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "ESC",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Gray),
                        ),
                        Span::styled(" to finish)", Style::default().fg(Color::DarkGray)),
                    ])),
            );
        f.render_widget(input, rects[1]);

        if self.mode == Mode::Insert {
            f.set_cursor(
                (rects[1].x + 1 + self.input.cursor() as u16).min(rects[1].x + rects[1].width - 2),
                rects[1].y + 1,
            )
        }

        if self.show_help {
            let rect = rect.inner(&Margin {
                horizontal: 4,
                vertical: 2,
            });
            f.render_widget(Clear, rect);
            let block = Block::default()
                .title(Line::from(vec![Span::styled(
                    "Key Bindings",
                    Style::default().add_modifier(Modifier::BOLD),
                )]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            f.render_widget(block, rect);
            let rows = vec![
                Row::new(vec!["j", "Increment"]),
                Row::new(vec!["k", "Decrement"]),
                Row::new(vec!["/", "Enter Input"]),
                Row::new(vec!["ESC", "Exit Input"]),
                Row::new(vec!["Enter", "Submit Input"]),
                Row::new(vec!["q", "Quit"]),
                Row::new(vec!["?", "Open Help"]),
            ];
            let table = Table::new(
                rows,
                [Constraint::Percentage(10), Constraint::Percentage(90)],
            )
            .header(
                Row::new(vec!["Key", "Action"])
                    .bottom_margin(1)
                    .style(Style::default().add_modifier(Modifier::BOLD)),
            )
            .column_spacing(1);
            f.render_widget(
                table,
                rect.inner(&Margin {
                    vertical: 4,
                    horizontal: 2,
                }),
            );
        }

        f.render_widget(
            Block::default().title(
                block::Title::from(format!(
                    "{:?}",
                    &self
                        .last_events
                        .iter()
                        .map(key_event_to_string)
                        .collect::<Vec<_>>()
                ))
                .alignment(Alignment::Right),
            ),
            Rect {
                x: rect.x + 1,
                y: rect.height.saturating_sub(1),
                width: rect.width.saturating_sub(2),
                height: 1,
            },
        );

        Ok(())
    }
}
