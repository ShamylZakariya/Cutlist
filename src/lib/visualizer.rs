use macroquad::prelude::*;

use super::solver;

const PADDING: f32 = 10f32;
const FONT_SIZE: f32 = 16f32;
const BOARD_COLOR: Color = Color::new(0f32, 0f32, 0f32, 0.1);
const BOARD_STROKE_COLOR: Color = Color::new(0f32, 0f32, 0f32, 0.2);

const CUT_COLOR: Color = Color::new(0.5f32, 0.5f32, 0.5f32, 1f32);
const CUT_STROKE_COLOR: Color = Color::new(0.25f32, 0.25f32, 0.25f32, 1f32);

const CROSSCUT_LINE_COLOR: Color = Color::new(1f32, 0f32, 0f32, 0.5);

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

fn render_board(board: &solver::Board, top_left: Vec2, scale: f32) -> Vec<Label> {
    let stroke_thickness = 1f32 / scale;
    let mut labels = Vec::new();

    // Draw the board
    draw_rectangle(
        top_left.x,
        top_left.y,
        board.length,
        board.width,
        BOARD_COLOR,
    );
    draw_rectangle_lines(
        top_left.x,
        top_left.y,
        board.length,
        board.width,
        stroke_thickness,
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
        for cut in &stack.cuts {
            draw_rectangle(
                stack_origin.x,
                stack_origin.y + cut_y,
                cut.length,
                cut.width,
                CUT_COLOR,
            );

            draw_rectangle_lines(
                stack_origin.x,
                stack_origin.y + cut_y,
                cut.length,
                cut.width,
                stroke_thickness,
                CUT_STROKE_COLOR,
            );

            labels.push(Label {
                text: cut.id.clone(),
                position: Vec2::new(stack_origin.x + cut.length / 2f32, stack_origin.y + cut_y + cut.width / 2f32),
                color: WHITE,
                anchor: LabelAnchor::Center,
            });

            cut_y += cut.width;
        }

        // draw the crosscut
        draw_line(
            stack_origin.x + stack.length(),
            top_left.y - (PADDING / 8f32),
            stack_origin.x + stack.length(),
            top_left.y + board.width + (PADDING / 8f32),
            2f32 * stroke_thickness,
            CROSSCUT_LINE_COLOR,
        );

        stack_origin.x += stack.length();
    }

    labels
}

fn draw_axis(at: Vec2, size: f32, color: Color) {
    draw_line(at.x, at.y - size, at.x, at.y + size, 1f32, color);
    draw_line(at.x - size, at.y, at.x + size, at.y, 1f32, color);
}

pub async fn show(cutlist: &[solver::Board]) {
    let mut scale = 16f32;
    let mut origin = Vec2::new(PADDING, PADDING);
    let mut mouse_down_position:Option<Vec2> = None;

    loop {
        clear_background(WHITE);

        draw_text("Cutlist", 20.0, screen_height() - 20., 16.0, DARKGRAY);

        draw_axis(origin, 10f32, GREEN);
        draw_axis(origin * scale, 10f32, DARKGREEN);

        push_camera_state();
        set_camera(&Camera2D::from_display_rect(Rect::new(
            0f32,
            0f32,
            screen_width() / scale,
            screen_height() / scale,
        )));

        let mut all_labels = Vec::new();
        let mut board_y_offset = 0f32;
        for board in cutlist {
            let mut board_labels = render_board(board, origin + Vec2::new(0f32, board_y_offset), scale);
            all_labels.append(&mut board_labels);
            board_y_offset += board.width + PADDING;
        }
        pop_camera_state();

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
                    ((label.position.y * scale) + (measure.height - measure.offset_y) * 0.5).floor(),
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
        let (mouse_x, mouse_y) = mouse_position();
        let left_mouse_down = is_mouse_button_down(MouseButton::Left);

        if mouse_wheel_y.abs() > 0f32 {
            let new_scale = (scale + (mouse_wheel_y * 1f32)).clamp(1f32, 64f32);
            let origin_offset = (origin * new_scale) - Vec2::new(mouse_x, mouse_y);
            let origin_offset = origin_offset * new_scale / scale;

            scale = new_scale;
            // origin += origin_offset / new_scale;

            println!("new_scale: {} origin_offset: {}", new_scale, origin_offset);
        }

        if left_mouse_down {
            if let Some(mouse_down_position) = mouse_down_position {
                let mouse_movement = Vec2::new(mouse_x, mouse_y) - mouse_down_position;
                origin += mouse_movement / scale;
            }
            mouse_down_position = Some(Vec2::new(mouse_x, mouse_y));
        } else {
            mouse_down_position = None;
        }

        if is_key_pressed(KeyCode::Space) {
            origin = Vec2::new(PADDING, PADDING);
            scale = 16f32;
        }

        next_frame().await
    }
}
