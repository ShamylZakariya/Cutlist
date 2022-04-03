use macroquad::prelude::*;

use super::solver::{self, Stack};

const PADDING: f32 = 10f32;
const FONT_SIZE: f32 = 16f32;
const BOARD_COLOR: Color = Color::new(0f32, 0f32, 0f32, 0.1);
const BOARD_STROKE_COLOR: Color = Color::new(0f32, 0f32, 0f32, 0.2);

const CUT_COLOR: Color = Color::new(0.5f32, 0.5f32, 0.5f32, 0.5f32);
const CUT_STROKE_COLOR: Color = Color::new(0.25f32, 0.25f32, 0.25f32, 1f32);

const PRIMARY_CROSSCUT_LINE_COLOR: Color = Color::new(1f32, 0f32, 0f32, 0.5);
const SECONDARY_CROSSCUT_LINE_COLOR: Color = Color::new(0f32, 1f32, 0f32, 0.5);

#[derive(Clone, Copy)]
enum LabelAnchor {
    Left,
    Center,
    Right,
}

#[derive(Clone)]
struct Label {
    text: String,
    position: Vec2,
    color: Color,
    anchor: LabelAnchor,
}

fn draw_rectangle_scaled(
    top_left: Vec2,
    size: Vec2,
    scale: f32,
    fill_color: Color,
    stroke_color: Color,
) {
    draw_rectangle(
        top_left.x * scale,
        top_left.y * scale,
        size.x * scale,
        size.y * scale,
        fill_color,
    );
    draw_rectangle_lines(
        top_left.x * scale,
        top_left.y * scale,
        size.x * scale,
        size.y * scale,
        1f32,
        stroke_color,
    );
}

fn draw_line_scaled(start: Vec2, end: Vec2, scale: f32, color: Color) {
    draw_line(
        start.x * scale,
        start.y * scale,
        end.x * scale,
        end.y * scale,
        1f32,
        color,
    );
}

fn render_board(board: &solver::Board, top_left: Vec2, scale: f32) -> Vec<Label> {
    let mut labels = Vec::new();

    // Draw the board
    draw_rectangle_scaled(
        top_left,
        Vec2::new(board.length, board.width),
        scale,
        BOARD_COLOR,
        BOARD_STROKE_COLOR,
    );
    labels.push(Label {
        text: format!("{} ({} by {})", board.id, board.length, board.width),
        position: top_left,
        color: BLACK,
        anchor: LabelAnchor::Left,
    });

    // Draw the cut stacks
    let mut stack_origin = top_left;
    for stack in &board.stacks {
        let mut cut_y = 0f32;
        for crosscut_stack in &stack.stacks {
            let mut cut_x = 0f32;

            for cut in &crosscut_stack.stack {
                draw_rectangle_scaled(
                    Vec2::new(stack_origin.x + cut_x, stack_origin.y + cut_y),
                    Vec2::new(cut.length, cut.width),
                    scale,
                    CUT_COLOR,
                    CUT_STROKE_COLOR,
                );

                // draw the crosscut
                draw_line_scaled(
                    Vec2::new(
                        stack_origin.x + cut_x,
                        stack_origin.y + cut_y - (PADDING / 16f32),
                    ),
                    Vec2::new(
                        stack_origin.x + cut_x,
                        stack_origin.y + cut_y + crosscut_stack.width() + (PADDING / 16f32),
                    ),
                    scale,
                    SECONDARY_CROSSCUT_LINE_COLOR,
                );

                labels.push(Label {
                    text: cut.id.clone(),
                    position: Vec2::new(
                        stack_origin.x + cut_x + cut.length / 2f32,
                        stack_origin.y + cut_y + cut.width / 2f32,
                    ),
                    color: WHITE,
                    anchor: LabelAnchor::Center,
                });

                cut_x += cut.length
            }

            cut_y += crosscut_stack.width()
        }

        // draw the crosscut
        draw_line_scaled(
            Vec2::new(
                stack_origin.x + stack.length(),
                top_left.y - (PADDING / 8f32),
            ),
            Vec2::new(
                stack_origin.x + stack.length(),
                top_left.y + board.width + (PADDING / 8f32),
            ),
            scale,
            PRIMARY_CROSSCUT_LINE_COLOR,
        );

        stack_origin.x += stack.length();
    }

    labels
}

fn draw_axis(at: Vec2, size: f32, color: Color) {
    draw_line(at.x, at.y - size, at.x, at.y + size, 1f32, color);
    draw_line(at.x - size, at.y, at.x + size, at.y, 1f32, color);
}

pub async fn show(solutions: &[Vec<solver::Board>], print: bool) {
    let mut scale = 16f32;
    let mut origin = Vec2::new(0f32, 0f32);
    let mut mouse_down_position: Option<Vec2> = None;
    let mut current_solution_index: usize = 0;

    if print {
        println!(
            "solution[{}]:\n{:#?}\n\n",
            current_solution_index, &solutions[current_solution_index]
        );
    }

    loop {
        let cutlist = &solutions[current_solution_index];

        clear_background(WHITE);

        draw_text(
            &format!(
                "Solution {} of {} (score: {})",
                current_solution_index + 1,
                solutions.len(),
                solver::score(cutlist)
            ),
            20.0,
            screen_height() - 20.,
            16.0,
            DARKGRAY,
        );

        draw_axis(origin * scale, 10f32, GREEN);

        let mut all_labels = Vec::new();
        let mut board_y_offset = 0f32;
        for board in cutlist {
            let mut board_labels =
                render_board(board, origin + Vec2::new(0f32, board_y_offset), scale);
            all_labels.append(&mut board_labels);
            board_y_offset += board.width + PADDING;
        }

        for label in &all_labels {
            let measure = measure_text(&label.text, None, FONT_SIZE as u16, 1f32);
            match label.anchor {
                LabelAnchor::Left => draw_text(
                    &label.text,
                    (label.position.x * scale).floor(),
                    ((label.position.y * scale) - measure.height * 0.25).floor(),
                    FONT_SIZE,
                    label.color,
                ),
                LabelAnchor::Center => draw_text(
                    &label.text,
                    ((label.position.x * scale) - measure.width * 0.5).floor(),
                    ((label.position.y * scale) + (measure.height - measure.offset_y) * 0.5)
                        .floor(),
                    FONT_SIZE,
                    label.color,
                ),
                LabelAnchor::Right => draw_text(
                    &label.text,
                    ((label.position.x * scale) - measure.width).floor(),
                    ((label.position.y * scale) - measure.height * 0.25).floor(),
                    FONT_SIZE,
                    label.color,
                ),
            };
        }

        // Input

        let (_, mouse_wheel_y) = mouse_wheel();
        let mouse_position = {
            let (mouse_x, mouse_y) = mouse_position();
            Vec2::new(mouse_x, mouse_y)
        };
        let left_mouse_down = is_mouse_button_down(MouseButton::Left);

        if mouse_wheel_y.abs() > 0f32 {
            let new_scale = (scale + (mouse_wheel_y * 2f32)).clamp(1f32, 64f32);
            let old_origin = origin * scale;
            let old_offset_to_cursor = old_origin - mouse_position;
            let new_offset_to_cursor = old_offset_to_cursor * new_scale / scale;
            let new_origin = mouse_position + new_offset_to_cursor;

            scale = new_scale;
            origin = new_origin / scale;
        }

        if left_mouse_down {
            if let Some(mouse_down_position) = mouse_down_position {
                let mouse_movement = mouse_position - mouse_down_position;
                origin += mouse_movement / scale;
            }
            mouse_down_position = Some(mouse_position);
        } else {
            mouse_down_position = None;
        }

        if is_key_pressed(KeyCode::Space) {
            origin = Vec2::new(0f32, 0f32);
            scale = 16f32;
        }

        let solution_index_changed: bool = if is_key_pressed(KeyCode::J) {
            current_solution_index = (current_solution_index + 1).min(solutions.len() - 1);
            true
        } else if is_key_pressed(KeyCode::K) && current_solution_index > 0 {
            current_solution_index -= 1;
            true
        } else {
            false
        };

        if print && solution_index_changed {
            println!(
                "solution[{}]:\n{:#?}\n\n",
                current_solution_index, &solutions[current_solution_index]
            );
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await
    }
}
