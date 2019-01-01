use rand::{thread_rng, Rng};

// use super::UI;

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
}

impl Cell {
    fn new() -> Cell {
        Cell {
            connections: 0,
            is_mined: false,
            is_open: false,
            is_flagged: false,
            is_question: false,
        }
    }

    pub fn rotate_marker(&mut self) {
        if !self.is_open && !self.is_flagged && !self.is_question {
            self.is_flagged = true;
        } else if self.is_flagged {
            self.is_flagged = false;
            self.is_question = true;
        } else if self.is_question {
            self.is_question = false;
        } else if self.is_open && (self.is_flagged || self.is_question) {
            self.is_flagged = false;
            self.is_question = false;
        }
    }
}

pub struct Game {
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

        self.expose(&start);
    }

    pub fn expose(&mut self, pos: &Point) {
        let cell = &mut self.grid[pos.x as usize][pos.y as usize];

        if cell.is_flagged || cell.is_question {
            return;
        }

        if !cell.is_open {
            cell.is_open = true;

            if cell.is_mined {
                self.lose_game();
            } else if cell.connections > 0 {
                return;
            }
        }

        NEIGHBORS.iter().for_each(|neigh| {
            let loc = Point {
                x: pos.x + neigh.x,
                y: pos.y + neigh.y,
            };

            if self.in_grid(&loc) {
                let cell = &mut self.grid[loc.x as usize][loc.y as usize];

                if cell.connections > 0 && !cell.is_flagged && !cell.is_question {
                    cell.is_open = true;

                    if cell.is_mined {
                        self.lose_game();
                    }
                } else if cell.connections == 0 && !cell.is_open {
                    self.expose(&loc);
                }
            }
        })
    }

    pub fn place_mine(&mut self, pos: &Point) {
        self.grid[pos.x as usize][pos.y as usize].is_mined = true;

        NEIGHBORS.iter().for_each(|neigh| {
            let loc = Point {
                x: neigh.x + pos.x,
                y: neigh.y + pos.y,
            };

            if self.in_grid(&loc) {
                self.grid[loc.x as usize][loc.y as usize].connections += 1;
            }
        })
    }

    pub fn lose_game(&mut self) {
        self.grid
            .iter_mut()
            .for_each(|row| row.iter_mut().for_each(|cell| cell.is_open = true));

        self.game_over = true;
    }

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
