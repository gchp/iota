extern crate rustbox;


pub struct UIBuffer {
    width: uint,
    height: uint,
    cells: Vec<Vec<Cell>>
}

impl UIBuffer {
    pub fn new(width: uint, height: uint) -> UIBuffer {
        let cells = Cell::create_grid(width, height, ' ');

        UIBuffer {
            width: width,
            height: height,
            cells: cells,
        }
    }

    pub fn draw_range(&self, start: uint, stop: uint) {
        let rows = self.cells.slice(start, stop);
        for row in rows.iter() {
            for cell in row.iter() {
                rustbox::print_char(cell.x, cell.y, rustbox::Style::Normal, cell.fg, cell.bg, cell.ch);
            }
        }
    }

    /// Recreated the entire grid, will cells containing `ch`.
    pub fn fill(&mut self, ch: char) {
        self.cells = Cell::create_grid(self.width, self.height, ch);
    }

    /// Update the `ch` attribute of an individual cell
    pub fn update_cell_content(&mut self, x: uint, y: uint, ch: char) {
        self.cells[y][x].ch = ch
    }

    /// Update the `ch`, `fg`, and `bg` attributes of an indivudual cell
    pub fn update_cell(&mut self, x: uint, y: uint, ch: char, fg: rustbox::Color, bg: rustbox::Color) {
        let cell = &mut self.cells[y][x];
        cell.ch = ch;
        cell.fg = fg;
        cell.bg = bg;
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

    pub fn create_grid(width: uint, height: uint, ch: char) -> Vec<Vec<Cell>> {
        let mut rows = Vec::new();
        for voffset in range(0, height) {
            let mut cells = Vec::new();
            for boffset in range(0, width) {
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

