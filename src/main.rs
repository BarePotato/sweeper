use rand::{thread_rng, Rng};
use sfml::graphics::{
    Color, Font, PrimitiveType, RectangleShape, RenderStates, RenderTarget, RenderWindow, Shape,
    Text, Transformable, Vertex,
};
use sfml::system::Vector2f;
use sfml::window::{mouse, Event, Style};

const DEF_GRID: (u32, u32) = (10, 10); // grid squares
const DEF_GRID_SQ: u32 = 60; // pixels per grid square
const DEF_FONT_SIZE: u32 = DEF_GRID_SQ / 2;
const DEF_MARGIN: u32 = 8; // pixels around edge of window
const DEF_CTRL_CANVAS: u32 = 128; // pixels on edge of x for buttons and stuff

const neighbors: &[Point] = &[
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

fn main() {
    //// Game stuffs
    let mut first_click = true;
    let mut click_grid = Point { x: -1, y: -1 };

    //// Basic Window and UI stuff
    let grid_w = DEF_GRID.0 * DEF_GRID_SQ;
    let grid_h = DEF_GRID.1 * DEF_GRID_SQ;
    let win_x = &grid_w + (DEF_MARGIN * 2) + DEF_CTRL_CANVAS;
    let win_y = &grid_h + (DEF_MARGIN * 2);
    let ctrl_canvas_left = win_x - 128;
    let grid_point = Point {
        x: DEF_GRID.0 as i8,
        y: DEF_GRID.1 as i8,
    };

    //// Our window
    let mut window = RenderWindow::new(
        (win_x, win_y),
        "Potato Sweeper!",
        Style::CLOSE,
        &Default::default(),
    );

    //// UI Font
    let ui_font = Font::from_file("courbd.ttf").unwrap();
    let mut my_text = Text::new("", &ui_font, 20);

    //// Playfield BG
    let mut my_rect = RectangleShape::with_size((grid_w as f32, grid_h as f32).into());
    my_rect.set_position((DEF_MARGIN as f32, DEF_MARGIN as f32));
    my_rect.set_fill_color(&Color::rgb(28, 36, 43));

    //// Control surface and Controls
    // line dividing gameplay area from controls
    let ctrl_canvas_edge = [
        Vertex::with_pos_color(
            Vector2f::new(ctrl_canvas_left as f32, 0.),
            Color::rgb(5, 11, 16),
        ),
        Vertex::with_pos_color(
            Vector2f::new(ctrl_canvas_left as f32, win_y as f32),
            Color::rgb(5, 11, 16),
        ),
        Vertex::with_pos_color(
            Vector2f::new((ctrl_canvas_left + 1) as f32, win_y as f32),
            Color::rgb(64, 71, 78),
        ),
        Vertex::with_pos_color(
            Vector2f::new((ctrl_canvas_left + 1) as f32, 0.),
            Color::rgb(64, 71, 78),
        ),
    ];

    //// starting grid
    let mut grid = vec![vec![Cell::new(); DEF_GRID.0 as usize]; DEF_GRID.1 as usize];

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::MouseButtonPressed { button, x, y } => {
                    let mouse = mouse_to_grid(x, y);
                    if in_grid(&mouse, &grid_point) {
                        // capture grid clicked on
                        click_grid = mouse;
                    }
                }
                Event::MouseButtonReleased { button, x, y } => {
                    let mouse_grid = mouse_to_grid(x, y);
                    let cell = &mut grid[mouse_grid.x as usize][mouse_grid.y as usize];

                    if in_grid(&mouse_grid, &grid_point) {
                        // check release grid vs click grid
                        if click_grid != mouse_grid {
                            continue;
                        }
                        match button {
                            mouse::Button::Left => {
                                if first_click {
                                    grid = generate_grid(&mouse_grid);
                                    first_click = false;
                                } else {
                                    if cell.is_mined {
                                        grid.iter_mut().for_each(|row| {
                                            row.iter_mut().for_each(|cell| cell.is_open = true)
                                        });
                                    } else if cell.connections > 0 {
                                        cell.is_open = true;
                                    } else if cell.connections == 0 {
                                        expose_open(&mut grid, &mouse_grid);
                                    }
                                }
                            }
                            mouse::Button::Middle => {
                                first_click = true;
                                grid = vec![
                                    vec![Cell::new(); DEF_GRID.0 as usize];
                                    DEF_GRID.1 as usize
                                ];
                            }
                            mouse::Button::Right => {}
                            _ => {}
                        }
                    }
                }
                Event::MouseMoved { x, y } => {
                    let loc = Point {
                        x: ((x - DEF_MARGIN as i32) / DEF_GRID_SQ as i32) as i8,
                        y: ((y - DEF_MARGIN as i32) / DEF_GRID_SQ as i32) as i8,
                    };
                    if in_grid(&mouse_to_grid(x, y), &grid_point) {
                        let mut loc_s = "";
                        if grid[loc.x as usize][loc.y as usize].is_mined {
                            loc_s = "X";
                        }
                        my_text.set_string(format!("{},{} : {}", loc.x, loc.y, loc_s).as_str());
                        my_text.set_position((
                            (ctrl_canvas_left + DEF_MARGIN) as f32,
                            DEF_MARGIN as f32,
                        ));
                    } else {
                        my_text.set_string("");
                    }
                }
                _ => {}
            }
        }

        window.set_active(true);
        // background color of window
        window.clear(&Color::rgb(1, 6, 9));
        // rectangle playfield
        window.draw_rectangle_shape(&my_rect, RenderStates::default());
        // line blocking off controls area
        window.draw_line(&ctrl_canvas_edge);
        window.draw_text(&my_text, RenderStates::default());
        // draw grid
        window.draw_grid(&grid);
        // draw all the things
        window.display();
    }
}

trait BareDraw {
    fn draw_line(&self, vertices: &[Vertex]);
    fn draw_grid(&self, grid: &Vec<Vec<Cell>>);
}

impl BareDraw for RenderWindow {
    fn draw_line(&self, vertices: &[Vertex]) {
        self.draw_primitives(&vertices, PrimitiveType::LineStrip, RenderStates::default());
    }

    fn draw_grid(&self, grid: &Vec<Vec<Cell>>) {
        let font = Font::from_file("courbd.ttf").unwrap();
        let mut text = Text::new("", &font, DEF_FONT_SIZE);
        text.set_fill_color(&Color::WHITE);

        for i in 0..DEF_GRID.0 {
            for j in 0..DEF_GRID.1 {
                let x = (i * DEF_GRID_SQ) + DEF_MARGIN + 1;
                let y = (j * DEF_GRID_SQ) + DEF_MARGIN;
                let w = &x + (DEF_GRID_SQ - 1);
                let h = &y + (DEF_GRID_SQ - 1);

                let mut my_rect = RectangleShape::with_size(
                    ((DEF_GRID_SQ - 1) as f32, (DEF_GRID_SQ - 1) as f32).into(),
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
                if grid[i as usize][j as usize].is_open {
                    self.draw_rectangle_shape(&my_rect, RenderStates::default());

                    let cell = &grid[i as usize][j as usize];

                    if cell.is_mined {
                        text.set_string("X");
                        text.set_fill_color(&Color::RED);
                    } else if cell.connections > 0u8 {
                        text.set_string(cell.connections.to_string().as_str());
                    } else {
                        text.set_string("");
                    }

                    if !text.string().is_empty() {
                        let x = (i * DEF_GRID_SQ) + (DEF_GRID_SQ / 2);
                        let y = (j * DEF_GRID_SQ) + (DEF_GRID_SQ / 2 / 2);
                        text.set_position((x as f32, y as f32));
                        self.draw_text(&text, RenderStates::default());
                    }
                }
            }
        }

        // expose_grid(&self, &grid);
    }
}

fn in_grid(point: &Point, grid: &Point) -> bool {
    if point.x < 0 || point.y < 0 || point.x >= grid.x || point.y >= grid.y {
        return false;
    }
    true
}

fn mouse_to_grid(mut x: i32, mut y: i32) -> Point {
    if x < DEF_MARGIN as i32 || y < DEF_MARGIN as i32 {
        x = -1;
        y = -1;
    } else {
        x = (x - DEF_MARGIN as i32) / DEF_GRID_SQ as i32;
        y = (y - DEF_MARGIN as i32) / DEF_GRID_SQ as i32;
    }
    Point {
        x: x as i8,
        y: y as i8,
    }
}

fn generate_grid(start: &Point) -> Vec<Vec<Cell>> {
    let mine_count = (DEF_GRID.0 * DEF_GRID.1) / 10;

    let (grid_w, grid_h) = (DEF_GRID.0 as usize, DEF_GRID.1 as usize);

    let mut rng = thread_rng();

    ////todo: make sure we aren't duplicating boards in memory
    let mut grid: Vec<Vec<Cell>> = vec![vec![Cell::new(); grid_w]; grid_h];

    for _ in 0..mine_count {
        let (mut x, mut y) = (rng.gen_range(0, grid_w), rng.gen_range(0, grid_h));

        while grid[x][y].is_mined || (x == start.x as usize && y == start.y as usize) {
            x = rng.gen_range(0, grid_w);
            y = rng.gen_range(0, grid_h);
        }

        place_mine(
            &mut grid,
            &Point {
                x: x as i8,
                y: y as i8,
            },
        );
    }

    // expose our starting grid
    if grid[start.x as usize][start.y as usize].connections == 0 {
        expose_open(&mut grid, &start);
    } else {
        grid[start.x as usize][start.y as usize].is_open = true;
    }

    grid
}

fn expose_open(mut grid: &mut Vec<Vec<Cell>>, pos: &Point) {
    grid[pos.x as usize][pos.y as usize].is_open = true;

    for (i, neighbor) in neighbors.iter().enumerate() {
        if i >= 4 {
            break;
        }

        let location = Point {
            x: pos.x + neighbor.x,
            y: pos.y + neighbor.y,
        };

        let grid_size = Point {
            x: DEF_GRID.0 as i8,
            y: DEF_GRID.1 as i8,
        };

        if in_grid(&location, &grid_size) {
            if grid[location.x as usize][location.y as usize].connections == 0
                && !grid[location.x as usize][location.y as usize].is_open
            {
                expose_open(&mut grid, &location);
            }
        }
    }
}

fn place_mine(mut grid: &mut Vec<Vec<Cell>>, place_grid: &Point) {
    let (x, y) = (place_grid.x as usize, place_grid.y as usize);

    // if !grid[x][y].is_mined {
    grid[x][y].is_mined = true;
    //todo: lose it?
    grid[x][y].position = Point {
        x: x as i8,
        y: y as i8,
    };

    for neighbor in neighbors {
        let location = Point {
            x: place_grid.x as i8 + neighbor.x,
            y: place_grid.y as i8 + neighbor.y,
        };

        let grid_size = Point {
            x: DEF_GRID.0 as i8,
            y: DEF_GRID.1 as i8,
        };
        if in_grid(&location, &grid_size) {
            grid[location.x as usize][location.y as usize].connections += 1;
        }
    }
    // }
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
