use rand::{thread_rng, Rng};
use sfml::graphics::{
    Color, Font, PrimitiveType, RectangleShape, RenderStates, RenderTarget, RenderWindow, Shape,
    Text, Transformable, Vertex,
};
use sfml::system::Vector2f;
use sfml::window::{mouse, Event, Style};

const DEF_CTRL_CANVAS: u32 = 128; // pixels on edge of x for buttons and stuff

const NEIGHBORS: &[Point] = &[
    Point { x: -1, y: 0 },
    Point { x: 1, y: 0 },
    Point { x: 0, y: -1 },
    Point { x: 0, y: 1 },
    Point { x: -1, y: 1 },
    Point { x: 1, y: 1 },
    Point { x: -1, y: -1 },
    Point { x: 1, y: -1 },
];

#[derive(Clone, Debug, PartialEq)]
struct Point {
    x: i8,
    y: i8,
}

#[derive(Clone, Debug)]
struct Cell {
    connections: u8,
    is_mined: bool,
    is_open: bool,
    is_flagged: bool,
    is_question: bool,
    position: Point,
    has_mouse: bool,
}

impl Cell {
    fn new() -> Cell {
        Cell {
            connections: 0,
            is_mined: false,
            is_open: false,
            is_flagged: false,
            is_question: false,
            position: Point { x: 0, y: 0 },
            has_mouse: false,
        }
    }
}

struct Game {
    grid: Vec<Vec<Cell>>,
    grid_width: usize,
    grid_height: usize,
    grid_square: usize,
    first_click: bool,
    game_over: bool,
    game_won: bool,
}

impl Game {
    fn new() -> Game {
        let grid_width = 10;
        let grid_height = 10;

        Game {
            grid: vec![vec![Cell::new(); grid_width]; grid_height],
            grid_width,
            grid_height,
            grid_square: 60,
            first_click: true,
            game_over: false,
            game_won: false,
        }
    }
}

struct UI {
    window: RenderWindow,
    margin: usize,
    font: Font,
    font_size: u32,
    control_surface_width: usize,
    control_surface_left: u32,
}

impl UI {
    fn new(game: &Game) -> UI {
        let margin = 8;
        let font_size = (game.grid_square as f32 / 1.5) as u32;
        let control_surface_width = 128;
        let ui_width =
            (game.grid_square * game.grid_width + (margin * 2) + control_surface_width) as u32;
        let ui_height = (game.grid_square * game.grid_height + (margin * 2)) as u32;
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

fn main() {
    //// Game stuffs
    let mut game = Game::new();
    //// Basic Window and UI stuff
    let mut ui = UI::new(&game);

    //// UI related something or other
    let mut click_grid = Point { x: -1, y: -1 };

    let grid_point = Point {
        x: game.grid_square as i8,
        y: game.grid_square as i8,
    };

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
                    if in_grid(&mouse, &mut game) {
                        // capture grid clicked on
                        click_grid = mouse;
                    }
                }
                Event::MouseButtonReleased { button, x, y } => {
                    let mouse_grid = mouse_to_grid(x, y, &game, &ui);

                    if in_grid(&mouse_grid, &mut game) {
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
                                } else if cell.is_flagged {
                                    continue;
                                } else if cell.is_mined {
                                    game.lose_game();
                                } else if cell.connections > 0 {
                                    if cell.is_open {
                                        &game.expose_neighbors(&mouse_grid);
                                    } else {
                                        if cell.is_flagged || cell.is_question {
                                            cell.is_open = false;
                                        } else {
                                            cell.is_open = true;
                                        }
                                    }
                                } else if cell.connections == 0 {
                                    &game.expose_open(&mouse_grid);
                                }
                            }
                            mouse::Button::Middle => game = Game::new(),
                            mouse::Button::Right => {
                                if game.first_click {
                                    continue;
                                } else {
                                    if cell.is_flagged {
                                        cell.is_flagged = false;
                                        cell.is_question = true;
                                    } else if cell.is_question {
                                        cell.is_flagged = false;
                                        cell.is_question = false;
                                    } else {
                                        cell.is_flagged = true;
                                        cell.is_question = false;
                                    }
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
                    if in_grid(mouse_loc, &mut game) {
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

trait BareDraw {
    fn draw_line(&self, vertices: &[Vertex]);
    fn draw_grid(&self, game: &mut Game, ui: &UI);
}

impl BareDraw for RenderWindow {
    fn draw_line(&self, vertices: &[Vertex]) {
        self.draw_primitives(&vertices, PrimitiveType::LineStrip, RenderStates::default());
    }

    // fn draw_ui(&self) {}

    fn draw_grid(&self, game: &mut Game, ui: &UI) {
        let grid = &mut game.grid;

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

                let cell = &grid[i as usize][j as usize];
                if cell.is_open {
                    self.draw_rectangle_shape(&my_rect, RenderStates::default());

                    if cell.is_mined {
                        text.set_string("X");
                        text.set_fill_color(&Color::RED);
                    } else if cell.connections > 0u8 {
                        text.set_string(cell.connections.to_string().as_str());
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

        // expose_grid(&self, &grid);
    }
}

fn in_grid(point: &Point, game: &mut Game) -> bool {
    if point.x < 0
        || point.y < 0
        || point.x >= game.grid_width as i8
        || point.y >= game.grid_height as i8
    {
        return false;
    }
    true
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

impl Game {
    fn generate_grid(&mut self, start: &Point) {
        let mine_count = (self.grid_width * self.grid_height) / 10;

        let (grid_w, grid_h) = (self.grid_width as usize, self.grid_height as usize);

        let mut rng = thread_rng();

        ////todo: make sure we aren't duplicating boards in memory
        self.grid = vec![vec![Cell::new(); grid_w]; grid_h];

        for _ in 0..mine_count {
            let (mut x, mut y) = (rng.gen_range(0, grid_w), rng.gen_range(0, grid_h));

            while self.grid[x][y].is_mined || (x == start.x as usize && y == start.y as usize) {
                x = rng.gen_range(0, grid_w);
                y = rng.gen_range(0, grid_h);
            }

            self.place_mine(&Point {
                x: x as i8,
                y: y as i8,
            });
        }

        // expose our starting grid
        if self.grid[start.x as usize][start.y as usize].connections == 0 {
            self.expose_open(&start);
        } else {
            self.grid[start.x as usize][start.y as usize].is_open = true;
        }
    }

    fn expose_open(&mut self, pos: &Point) {
        self.grid[pos.x as usize][pos.y as usize].is_open = true;

        for (i, neighbor) in NEIGHBORS.iter().enumerate() {
            // if i >= 4 {
            //     break;
            // }

            let location = Point {
                x: pos.x + neighbor.x,
                y: pos.y + neighbor.y,
            };

            let grid_size = Point {
                x: self.grid_width as i8,
                y: self.grid_height as i8,
            };

            if in_grid(&location, self) {
                let cell = &mut self.grid[location.x as usize][location.y as usize];

                if cell.connections == 0 && !cell.is_open {
                    self.expose_open(&location);
                } else if cell.connections > 0 && !cell.is_flagged && !cell.is_question {
                    cell.is_open = true;
                }
            }
        }
    }
//TODO: FIXME: See if we can merge expose_neighbors && expose_open
    fn expose_neighbors(&mut self, pos: &Point) {
        for neighbor in NEIGHBORS.iter() {
            let location = Point {
                x: pos.x + neighbor.x,
                y: pos.y + neighbor.y,
            };

            if self.in_grid(&location) {
                let cell = self.grid[location.x as usize][location.y as usize].clone();

                if cell.is_flagged || cell.is_question {
                    self.grid[location.x as usize][location.y as usize].is_open = false;
                } else if !cell.is_flagged && !cell.is_question && !cell.is_mined {
                    self.grid[location.x as usize][location.y as usize].is_open = true;
                } else if cell.is_mined && !cell.is_flagged && !cell.is_question {
                    self.lose_game();
                } else if cell.connections == 0 {
                    self.expose_open(&location);
                }
            }
        }
    }

    fn lose_game(&mut self) {
        self.grid
            .iter_mut()
            .for_each(|row| row.iter_mut().for_each(|cell| cell.is_open = true));

        self.game_over = true;
    }

    fn place_mine(&mut self, place_grid: &Point) {
        let (x, y) = (place_grid.x as usize, place_grid.y as usize);

        // if !grid[x][y].is_mined {
        self.grid[x][y].is_mined = true;
        //todo: lose it?
        self.grid[x][y].position = Point {
            x: x as i8,
            y: y as i8,
        };

        for neighbor in NEIGHBORS {
            let location = Point {
                x: place_grid.x as i8 + neighbor.x,
                y: place_grid.y as i8 + neighbor.y,
            };

            let grid_size = Point {
                x: self.grid_width as i8,
                y: self.grid_height as i8,
            };
            if in_grid(&location, self) {
                self.grid[location.x as usize][location.y as usize].connections += 1;
            }
        }
        // }
    }

    fn in_grid(&self, point: &Point) -> bool {
        if point.x < 0
            || point.y < 0
            || point.x >= self.grid_width as i8
            || point.y >= self.grid_height as i8
        {
            return false;
        }
        true
    }
}
//// DEBUG PURPOSES
// fn expose_grid(win: &RenderWindow, grid: &Vec<Vec<Cell>>) {
//     let font = Font::from_file("courbd.ttf").unwrap();
//     let mut text = Text::new("", &font, DEF_FONT_SIZE);
//     text.set_fill_color(&Color::WHITE);

//     for (i, row) in grid.iter().enumerate() {
//         for (j, cell) in row.iter().enumerate() {
//             text.set_fill_color(&Color::WHITE);

//             // let x = (i * DEF_GRID_SQ as usize) + (DEF_GRID_SQ as usize / 2)
//             //     - (DEF_FONT_SIZE / 2) as usize
//             //     + (DEF_MARGIN / 2) as usize;
//             // let y = (j * DEF_GRID_SQ as usize)
//             //     + ((DEF_GRID_SQ as usize / 2) + (DEF_MARGIN / 2) as usize);
//             let x = (i * DEF_GRID_SQ as usize) + (DEF_GRID_SQ / 2) as usize;
//             let y = (j * DEF_GRID_SQ as usize) + (DEF_GRID_SQ / 2 / 2) as usize;

//             if cell.is_mined {
//                 text.set_string("X");
//                 text.set_fill_color(&Color::RED);
//             } else if cell.connections > 0u8 {
//                 text.set_string(cell.connections.to_string().as_str());
//             } else {
//                 text.set_string("");
//             }

//             if !text.string().is_empty() {
//                 text.set_position((x as f32, y as f32));
//                 win.draw_text(&text, RenderStates::default());
//             }
//         }
//     }
// }
