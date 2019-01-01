use rand::{thread_rng, Rng};

use super::UI;

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
pub struct Point {
    pub x: i8,
    pub y: i8,
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub connections: u8,
    pub is_mined: bool,
    pub is_open: bool,
    pub is_flagged: bool,
    pub is_question: bool,
    pub position: Point,
    pub has_mouse: bool,
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

pub struct Game {
    pub ui: UI,
    pub grid: Vec<Vec<Cell>>,
    pub grid_width: usize,
    pub grid_height: usize,
    pub mine_percent: usize,
    pub grid_square: usize,
    pub first_click: bool,
    pub game_over: bool,
    pub game_won: bool,
}

impl Game {
    pub fn new() -> Game {
        let grid_width = 10;
        let grid_height = 10;
        let grid_square = 60;

        Game {
            ui: UI::new(grid_width, grid_height, grid_square),
            grid: vec![vec![Cell::new(); grid_width]; grid_height],
            grid_width,
            grid_height,
            mine_percent: 15,
            grid_square,
            first_click: true,
            game_over: false,
            game_won: false,
        }
    }

    pub fn generate_grid(&mut self, start: &Point) {
        let mine_count = ((self.grid_width * self.grid_height) as f64
            * (self.mine_percent as f64 / 100.)) as usize;

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

    pub fn expose(&mut self, pos: &Point) {
        let upos = (pos.x as usize, pos.y as usize);
        let cell = &mut self.grid[upos.0][upos.1];

        if cell.is_flagged || cell.is_question {
            return println!("Cell is Flagged/Question.");
        }

        if !cell.is_open {
            cell.is_open = true;

            if cell.is_mined {
                self.lose_game();
            } else if cell.connections > 0 {
                // Is open, Not mined, Not marked, Has connects
                return println!("Has connections, not opening more.");
            } else {
                // Not mined, 0 Connects, Not marked, Is open
            }
        } else {
            // Is open, Not marked, Not mined, ?? Connects
        }

        for neighbor in NEIGHBORS.iter() {
            let loc = Point {
                x: pos.x + neighbor.x,
                y: pos.y + neighbor.y,
            };

            if self.in_grid(&loc) {
                let cell = &mut self.grid[loc.x as usize][loc.y as usize];

                if cell.connections > 0 && !cell.is_flagged && !cell.is_question {
                    cell.is_open = true;
                } else if cell.connections == 0 && !cell.is_open {
                    self.expose(&loc);
                }
            }
        }
    }

    pub fn expose_open(&mut self, pos: &Point) {
        self.grid[pos.x as usize][pos.y as usize].is_open = true;

        for (i, neighbor) in NEIGHBORS.iter().enumerate() {
            let location = Point {
                x: pos.x + neighbor.x,
                y: pos.y + neighbor.y,
            };

            let grid_size = Point {
                x: self.grid_width as i8,
                y: self.grid_height as i8,
            };

            if self.in_grid(&location) {
                let cell = &mut self.grid[location.x as usize][location.y as usize];

                if cell.connections == 0 && !cell.is_open {
                    self.expose_open(&location);
                } else if cell.connections > 0 && !cell.is_flagged && !cell.is_question {
                    cell.is_open = true;
                }
            }
        }
    }
    //TODO: merge expose_neighbors && expose_open
    pub fn expose_neighbors(&mut self, pos: &Point) {
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
                    if cell.connections == 0 {
                        self.expose_open(&location);
                    }
                } else if cell.is_mined && !cell.is_flagged && !cell.is_question {
                    self.lose_game();
                } else if cell.connections == 0 {
                    self.expose_open(&location);
                }
            }
        }
    }

    pub fn lose_game(&mut self) {
        self.grid
            .iter_mut()
            .for_each(|row| row.iter_mut().for_each(|cell| cell.is_open = true));

        self.game_over = true;
    }

    pub fn place_mine(&mut self, place_grid: &Point) {
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
            if self.in_grid(&location) {
                self.grid[location.x as usize][location.y as usize].connections += 1;
            }
        }
        // }
    }

    // this needs to be a bounds check on a rect defining the grid area
    pub fn in_grid(&self, point: &Point) -> bool {
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
