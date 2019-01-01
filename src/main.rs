use sfml::graphics::{
    Color, RectangleShape, RenderStates, RenderTarget, Shape, Text, Transformable, Vertex,
};
use sfml::system::Vector2f;
use sfml::window::{mouse, Event};

mod game;
mod ui;

use crate::game::{Game, Point};
use crate::ui::{BareDraw, UI};

fn main() {
    //// Game stuffs
    let mut game = Game::new();
    //// Basic Window and UI stuff
    let mut ui = UI::new(game.grid_width, game.grid_height, game.grid_square);

    //// UI related something or other
    let mut click_grid = Point { x: -1, y: -1 };

    //// UI Font
    let mut my_text = Text::new("", &ui.font, ui.font_size / 2);

    //// Playfield BG
    let mut my_rect = RectangleShape::with_size(
        (
            (game.grid_square * game.grid_width) as f32,
            (game.grid_square * game.grid_height) as f32,
        )
            .into(),
    );
    my_rect.set_position((ui.margin as f32, ui.margin as f32));
    my_rect.set_fill_color(&Color::rgb(28, 36, 43));

    //// Control surface and Controls
    // line dividing gameplay area from controls
    let ctrl_canvas_edge = [
        Vertex::with_pos_color(
            Vector2f::new(ui.control_surface_left as f32, 0.),
            Color::rgb(5, 11, 16),
        ),
        Vertex::with_pos_color(
            Vector2f::new(ui.control_surface_left as f32, ui.window.size().y as f32),
            Color::rgb(5, 11, 16),
        ),
        Vertex::with_pos_color(
            Vector2f::new(
                (ui.control_surface_left + 1) as f32,
                ui.window.size().y as f32,
            ),
            Color::rgb(64, 71, 78),
        ),
        Vertex::with_pos_color(
            Vector2f::new((ui.control_surface_left + 1) as f32, 0.),
            Color::rgb(64, 71, 78),
        ),
    ];

    while ui.window.is_open() {
        while let Some(event) = ui.window.poll_event() {
            match event {
                Event::Closed => ui.window.close(),
                Event::MouseButtonPressed { button, x, y } => {
                    let mouse = mouse_to_grid(x, y, &game, &ui);
                    if game.in_grid(&mouse) {
                        // capture grid clicked on
                        click_grid = mouse;
                    }
                }
                Event::MouseButtonReleased { button, x, y } => {
                    let mouse_grid = mouse_to_grid(x, y, &game, &ui);

                    if game.in_grid(&mouse_grid) {
                        // check release grid vs click grid
                        if click_grid != mouse_grid {
                            continue;
                        }
                        let cell = &mut game.grid[mouse_grid.x as usize][mouse_grid.y as usize];
                        match button {
                            mouse::Button::Left => {
                                if game.first_click {
                                    &game.generate_grid(&mouse_grid);
                                    game.first_click = false;
                                } else {
                                    &game.expose(&mouse_grid);
                                }
                            }
                            mouse::Button::Middle => game = Game::new(),
                            mouse::Button::Right => {
                                if game.first_click {
                                    continue;
                                } else {
                                    cell.rotate_marker();
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::MouseMoved { x, y } => {
                    let loc = Point {
                        x: ((x - ui.margin as i32) / game.grid_square as i32) as i8,
                        y: ((y - ui.margin as i32) / game.grid_square as i32) as i8,
                    };
                    let mouse_loc = &mouse_to_grid(x, y, &game, &ui);
                    if game.in_grid(mouse_loc) {
                        let mut loc_s = "";
                        if game.grid[mouse_loc.x as usize][mouse_loc.y as usize].is_mined {
                            loc_s = "X";
                        }
                        my_text.set_string(format!("{},{} : {}", loc.x, loc.y, loc_s).as_str());
                        my_text.set_position((
                            (ui.control_surface_left + ui.margin as u32) as f32,
                            ui.margin as f32,
                        ));
                    } else {
                        my_text.set_string("");
                    }
                }
                _ => {}
            }
        }

        ui.window.set_active(true);
        // background color of window
        ui.window.clear(&Color::rgb(1, 6, 9));
        // rectangle playfield
        ui.window
            .draw_rectangle_shape(&my_rect, RenderStates::default());
        // line blocking off controls area
        ui.window.draw_line(&ctrl_canvas_edge);
        ui.window.draw_text(&my_text, RenderStates::default());
        // draw grid
        ui.window.draw_grid(&mut game, &ui);
        // draw all the things
        ui.window.display();
    }
}

fn mouse_to_grid(mut x: i32, mut y: i32, game: &Game, ui: &UI) -> Point {
    if x < ui.margin as i32 || y < ui.margin as i32 {
        x = -1;
        y = -1;
    } else {
        x = (x - ui.margin as i32) / game.grid_square as i32;
        y = (y - ui.margin as i32) / game.grid_square as i32;
    }
    Point {
        x: x as i8,
        y: y as i8,
    }
}
