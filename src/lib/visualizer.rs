use macroquad::prelude::*;

use super::solver;

const SCALE: f32 = 8f32;
const PADDING: f32 = 10f32;
const BOARD_COLOR: Color = Color::new(0f32, 0f32, 0f32, 0.1);
const BOARD_STROKE_COLOR: Color = Color::new(0f32, 0f32, 0f32, 0.2);

const CUT_COLOR: Color = Color::new(0.5f32, 0.5f32, 0.5f32, 1f32);
const CUT_STROKE_COLOR: Color = Color::new(0.25f32, 0.25f32, 0.25f32, 1f32);

const CROSSCUT_LINE_COLOR: Color = Color::new(1f32, 0f32, 0f32, 0.5);

fn render_board(board: &solver::Board, top_left: Vec2) {
    let stroke_thickness = 1f32 / SCALE;
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

            cut_y += cut.width;
        }

        // draw the crosscut
        draw_line(
            stack_origin.x + stack.length(),
            stack_origin.y - PADDING / 4f32,
            stack_origin.x + stack.length(),
            stack_origin.y + stack.width() + PADDING / 4f32,
            2f32 * stroke_thickness,
            CROSSCUT_LINE_COLOR,
        );

        stack_origin.x += stack.length();
    }
}

pub async fn show(cutlist: &[solver::Board]) {
    loop {
        clear_background(WHITE);

        draw_text("Cutlist", 20.0, screen_height() - 20., 16.0, DARKGRAY);

        push_camera_state();
        set_camera(&Camera2D::from_display_rect(Rect::new(
            0f32,
            0f32,
            screen_width() / SCALE,
            screen_height() / SCALE,
        )));

        let mut top_left = Vec2::new(PADDING, PADDING);
        for board in cutlist {
            render_board(board, top_left);
            top_left.y += board.width + PADDING;
        }
        pop_camera_state();

        next_frame().await
    }
}
