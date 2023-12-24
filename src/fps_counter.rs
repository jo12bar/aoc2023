use std::time::Instant;

use ratatui::{prelude::*, widgets::*};

use crate::{message::Message, model::Model, tui::Frame};

#[derive(Debug)]
pub struct FpsCounterModel {
    app_start_time: Instant,
    app_frames: u32,
    app_fps: f32,

    render_start_time: Instant,
    render_frames: u32,
    render_fps: f32,
}

impl Default for FpsCounterModel {
    fn default() -> Self {
        Self {
            app_start_time: Instant::now(),
            app_frames: 0,
            app_fps: 0.0,
            render_start_time: Instant::now(),
            render_frames: 0,
            render_fps: 0.0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FpsCounterMessage {
    Render,
    Tick,
}

pub fn update(model: &mut Model, msg: FpsCounterMessage) -> Option<Message> {
    let fps_model = &mut model.fps_counter;
    match msg {
        FpsCounterMessage::Render => {
            fps_model.app_frames += 1;
            let now = Instant::now();
            let elapsed = (now - fps_model.app_start_time).as_secs_f32();
            if elapsed >= 1.0 {
                fps_model.app_fps = fps_model.app_frames as f32 / elapsed;
                fps_model.app_start_time = now;
                fps_model.app_frames = 0;
            }

            None
        },

        FpsCounterMessage::Tick => {
            fps_model.render_frames += 1;
            let now = Instant::now();
            let elapsed = (now - fps_model.render_start_time).as_secs_f32();
            if elapsed >= 1.0 {
                fps_model.render_fps = fps_model.render_frames as f32 / elapsed;
                fps_model.render_start_time = now;
                fps_model.render_frames = 0;
            }

            None
        },
    }
}

pub fn view(model: &mut Model, f: &mut Frame, area: Rect) {
    f.render_widget(
        Paragraph::new(
            format!(
                "{:.02}fps, {:.02}tps",
                model.fps_counter.render_fps, model.fps_counter.app_fps
            )
            .fg(Color::DarkGray),
        ),
        area,
    );
}
