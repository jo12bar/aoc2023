use ratatui::{prelude::*, widgets::*};

use crate::{model::Model, tui::Frame};

pub fn view(model: &mut Model, f: &mut Frame) {
    f.render_widget(
        Paragraph::new(format!("Counter: {}", model.counter)),
        f.size(),
    );
}
