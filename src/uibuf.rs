extern crate rustbox;

use utils;

pub struct UIBuffer {
    cells: Vec<Vec<Cell>>
}

impl UIBuffer {
    pub fn new() -> UIBuffer {
        let cells = Cell::create_grid(' ');

        UIBuffer {
            cells: cells,
        }
    }

    pub fn draw_range(&self, start: uint, stop: uint) {
        let rows = self.cells.slice(start, stop);
        for row in rows.iter() {
            for cell in row.iter() {
                rustbox::print(cell.x, cell.y, rustbox::Normal, cell.fg, cell.bg, String::from_char(1, cell.ch));
            }
        }
    }

    /// Recreated the entire grid, will cells containing `ch`.
    pub fn fill(&mut self, ch: char) {
        self.cells = Cell::create_grid(ch);
    }

    /// Update the `ch` attribute of an individual cell
    pub fn update_cell_content(&mut self, x: uint, y: uint, ch: char) {
        self.cells[y][x].ch = ch
    }

    /// Update the `ch`, `fg`, and `bg` attributes of an indivudual cell
    pub fn update_cell(&mut self, x: uint, y: uint, ch: char, fg: rustbox::Color, bg: rustbox::Color) {
        // TODO(greg): refactor this to only look up the cell once
        self.cells[y][x].ch = ch;
        self.cells[y][x].fg = fg;
        self.cells[y][x].bg = bg;
    }
}


pub struct Cell {
    pub bg: rustbox::Color,
    pub fg: rustbox::Color,
    pub ch: char,
    pub x: uint,
    pub y: uint,
}


impl Cell {
    pub fn new() -> Cell {
        Cell {
            bg: rustbox::Color::Default,
            fg: rustbox::Color::White,
            ch: ' ',
            x: 0,
            y: 0,
        }
    }

    pub fn create_grid(ch: char) -> Vec<Vec<Cell>> {
        let term_height = utils::get_term_height();
        let term_width = utils::get_term_width();

        let mut rows = Vec::new();
        for voffset in range(0, term_height) {
            let mut cells = Vec::new();
            for boffset in range(0, term_width) {
                let mut cell = Cell::new();
                cell.x = boffset;
                cell.y = voffset;
                cell.ch = ch;
                cells.push(cell);
            }
            rows.push(cells);
        }

        return rows
    }
}

