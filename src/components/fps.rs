use std::time::Instant;

use color_eyre::eyre;
use ratatui::{prelude::*, widgets::*};

use super::{Component, ComponentError};
use crate::{action::Action, tui::Frame};

#[derive(Debug, Clone, PartialEq)]
pub enum Ticker {
    AppTick,
    RenderTick,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FpsCounter {
    app_start_time: Instant,
    app_frames: u32,
    app_fps: f32,

    render_start_time: Instant,
    render_frames: u32,
    render_fps: f32,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            app_start_time: Instant::now(),
            app_frames: 0,
            app_fps: 0.0,
            render_start_time: Instant::now(),
            render_frames: 0,
            render_fps: 0.0,
        }
    }

    fn app_tick(&mut self) -> eyre::Result<()> {
        self.app_frames += 1;
        let now = Instant::now();
        let elapsed = (now - self.app_start_time).as_secs_f32();
        if elapsed >= 1.0 {
            self.app_fps = self.app_frames as f32 / elapsed;
            self.app_start_time = now;
            self.app_frames = 0;
        }
        Ok(())
    }

    fn render_tick(&mut self) -> eyre::Result<()> {
        self.render_frames += 1;
        let now = Instant::now();
        let elapsed = (now - self.render_start_time).as_secs_f32();
        if elapsed >= 1.0 {
            self.render_fps = self.render_frames as f32 / elapsed;
            self.render_start_time = now;
            self.render_frames = 0;
        }
        Ok(())
    }
}

impl Component for FpsCounter {
    fn update(&mut self, action: Action) -> Result<Option<Action>, ComponentError> {
        match action {
            Action::Tick => {
                self.app_tick()?;
            },
            Action::Render => {
                self.render_tick()?;
            },
            _ => {},
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<(), ComponentError> {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // first row
                Constraint::Min(0),
            ])
            .split(area);

        let rect = rects[0];

        let s = format!(
            "{:.02} ticks per sec (app), {:.02} frames per sec (render)",
            self.app_fps, self.render_fps
        );
        let block = Block::default().title(block::Title::from(s.dim()).alignment(Alignment::Right));
        f.render_widget(block, rect);
        Ok(())
    }
}
