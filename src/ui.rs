use sfml::graphics::{
    Color, Font, PrimitiveType, RectangleShape, RenderStates, RenderTarget, RenderWindow, Shape,
    Text, Transformable, Vertex,
};
use sfml::system::Vector2f;
use sfml::window::Style;

use super::Game;

pub struct UI {
    pub window: RenderWindow,
    pub margin: usize,
    pub font: Font,
    pub font_size: u32,
    pub control_surface_width: usize,
    pub control_surface_left: u32,
}

impl UI {
    pub fn new(grid_width: usize, grid_height: usize, grid_square: usize) -> UI {
        let margin = 8;
        let font_size = (grid_square as f32 / 1.5) as u32;
        let control_surface_width = 128;
        let ui_width = (grid_square * grid_width + (margin * 2) + control_surface_width) as u32;
        let ui_height = (grid_square * grid_height + (margin * 2)) as u32;
        let control_surface_left = ui_width - control_surface_width as u32;

        UI {
            window: RenderWindow::new(
                (ui_width, ui_height),
                "Potato Sweeper!",
                Style::CLOSE,
                &Default::default(),
            ),
            margin,
            font: Font::from_file("courbd.ttf").unwrap(),
            font_size,
            control_surface_width,
            control_surface_left,
        }
    }
}

pub trait BareDraw {
    fn draw_line(&self, vertices: &[Vertex]);
    fn draw_grid(&self, game: &mut Game, ui: &UI);
}

impl BareDraw for RenderWindow {
    fn draw_line(&self, vertices: &[Vertex]) {
        self.draw_primitives(&vertices, PrimitiveType::LineStrip, RenderStates::default());
    }

    // fn draw_ui(&self) {}

    fn draw_grid(&self, game: &mut Game, ui: &UI) {
        let font = Font::from_file("courbd.ttf").unwrap();
        let mut text = Text::new("", &font, ui.font_size);
        text.set_outline_thickness(1.);
        text.set_outline_color(&Color::BLACK);

        for i in 0..game.grid_width {
            for j in 0..game.grid_height {
                let x = (i * game.grid_square) + ui.margin + 1;
                let y = (j * game.grid_square) + ui.margin;
                let w = &x + (game.grid_square - 1);
                let h = &y + (game.grid_square - 1);

                if game.game_over {
                    text.set_fill_color(&Color::RED);
                } else {
                    text.set_fill_color(&Color::WHITE);
                }

                let mut my_rect = RectangleShape::with_size(
                    ((game.grid_square - 1) as f32, (game.grid_square - 1) as f32).into(),
                );
                my_rect.set_position(((x - 1) as f32, y as f32));
                my_rect.set_fill_color(&Color::rgb(15, 20, 24));

                let top_left = [
                    Vertex::with_pos_color(
                        Vector2f::new(x as f32, h as f32),
                        Color::rgb(64, 71, 78),
                    ),
                    Vertex::with_pos_color(
                        Vector2f::new(x as f32, y as f32),
                        Color::rgb(64, 71, 78),
                    ),
                    Vertex::with_pos_color(
                        Vector2f::new((w - 1) as f32, y as f32),
                        Color::rgb(64, 71, 78),
                    ),
                ];
                let bot_right = [
                    Vertex::with_pos_color(
                        Vector2f::new(x as f32, h as f32),
                        Color::rgb(5, 11, 16),
                    ),
                    Vertex::with_pos_color(
                        Vector2f::new(w as f32, h as f32),
                        Color::rgb(5, 11, 16),
                    ),
                    Vertex::with_pos_color(
                        Vector2f::new(w as f32, (y + 1) as f32),
                        Color::rgb(5, 11, 16),
                    ),
                ];

                self.draw_line(&bot_right);
                self.draw_line(&top_left);

                let cell = &game.grid[i as usize][j as usize];
                if cell.is_open {
                    self.draw_rectangle_shape(&my_rect, RenderStates::default());

                    if cell.is_mined {
                        text.set_string("X");
                        text.set_fill_color(&Color::RED);
                    } else if cell.connections > 0u8 {
                        text.set_string(cell.connections.to_string().as_str());

                        match cell.connections {
                            1 => text.set_fill_color(&BareColor::BLUE),
                            2 => text.set_fill_color(&BareColor::DARK_GREEN),
                            3 => text.set_fill_color(&BareColor::RED),
                            4 => text.set_fill_color(&BareColor::DARK_BLUE),
                            5 => text.set_fill_color(&BareColor::DARK_RED),
                            6 => text.set_fill_color(&BareColor::DARK_CYAN),
                            7 => text.set_fill_color(&Color::BLACK),
                            8 => text.set_fill_color(&BareColor::GRAY),
                            _ => text.set_fill_color(&Color::WHITE),
                        }
                    } else {
                        text.set_string("");
                    }
                } else {
                    if cell.is_flagged {
                        text.set_string("F");
                        text.set_fill_color(&Color::BLUE);
                    } else if cell.is_question {
                        text.set_string("?");
                        text.set_fill_color(&Color::BLACK);
                    } else {
                        text.set_string("");
                    }
                }

                if !text.string().is_empty() {
                    let x = (i * game.grid_square) + (ui.font_size as usize / 2) + ui.margin;
                    let y = (j * game.grid_square) + (ui.font_size as usize / 2 / 2);
                    text.set_position((x as f32, y as f32));
                    self.draw_text(&text, RenderStates::default());
                }
            }
        }
    }
}

pub trait BareColor {
    const BLUE: Self;
    const DARK_BLUE: Self;
    const RED: Self;
    const DARK_RED: Self;
    const DARK_GREEN: Self;
    const CYAN: Self;
    const DARK_CYAN: Self;
    const GRAY: Self;
    const DARK_GRAY: Self;
}

impl BareColor for Color {
    const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 192,
    };
    const DARK_BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 128,
        a: 255,
    };
    const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 192,
    };
    const DARK_RED: Self = Self {
        r: 128,
        g: 0,
        b: 0,
        a: 255,
    };
    const DARK_GREEN: Self = Self {
        r: 0,
        g: 128,
        b: 0,
        a: 255,
    };
    const CYAN: Self = Self {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    };
    const DARK_CYAN: Self = Self {
        r: 0,
        g: 128,
        b: 128,
        a: 255,
    };
    const GRAY: Self = Self {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    };
    const DARK_GRAY: Self = Self {
        r: 64,
        g: 64,
        b: 64,
        a: 255,
    };
}
