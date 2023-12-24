use ratatui::{prelude::*, widgets::*};

use crate::{fps_counter, model::Model, tui::Frame};

pub fn view(model: &mut Model, f: &mut Frame) {
    let rects = Layout::new(
        Direction::Vertical,
        [Constraint::Percentage(100), Constraint::Min(3)],
    )
    .split(f.size());

    let counter_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().dim());

    f.render_widget(
        Paragraph::new(format!("Counter: {}", model.counter)),
        counter_block.inner(rects[0]),
    );

    f.render_widget(counter_block, rects[0]);

    let rects = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(100), // usage
            Constraint::Min(20),         // "30.00fps, 30.00tps" = 18 characters + 2 for border
        ],
    )
    .split(rects[1]);

    // Render usage
    let usage_block = Block::default()
        .title(block::Title::from("Usage").alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().dim());

    f.render_widget(
        Paragraph::new(Line::from(vec![
            "j".bold().fg(Color::Gray),
            " to increment, ".fg(Color::DarkGray),
            "k".bold().fg(Color::Gray),
            " to decrement, ".fg(Color::DarkGray),
            "q".bold().fg(Color::Gray),
            " to quit.".fg(Color::DarkGray),
        ])),
        usage_block.inner(rects[0]),
    );

    f.render_widget(usage_block, rects[0]);

    // Render fps/tps
    let fps_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().dim());

    fps_counter::view(model, f, fps_block.inner(rects[1]));
    f.render_widget(fps_block, rects[1]);
}
