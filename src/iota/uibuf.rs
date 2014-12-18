use rustbox::{Color, RustBox, Style};

pub struct UIBuffer {
    width: uint,
    height: uint,
    rows: Vec<Vec<Cell>>
}

impl UIBuffer {
    pub fn new(width: uint, height: uint) -> UIBuffer {
        let rows = Cell::create_grid(width, height, ' ');

        UIBuffer {
            width: width,
            height: height,
            rows: rows,
        }
    }

    pub fn draw_range(&mut self, rb: &RustBox, start: uint, stop: uint) {
        let rows = self.rows.slice_mut(start, stop);
        for row in rows.iter_mut() {
            for cell in row.iter_mut().filter(|cell| cell.dirty) {
                rb.print_char(cell.x, cell.y, Style::empty(), cell.fg, cell.bg, cell.ch);
                cell.dirty = false;
            }
        }
    }

    pub fn draw_everything(&mut self, rb: &RustBox) {
        let height = self.height;
        self.draw_range(rb, 0, height);
    }

    pub fn get_width(&self) -> uint {
        self.width
    }

    pub fn get_height(&self) -> uint {
        self.height
    }

    /// Set all cells to `ch`.
    pub fn fill(&mut self, ch: char) {
        for row in range(0, self.height) {
            for col in range(0, self.width) {
                self.update_cell_content(col, row, ch);
            }
        }
    }

    /// Update the `ch` attribute of an individual cell
    pub fn update_cell_content(&mut self, cell_num: uint, row_num: uint, ch: char) {
        self.rows[row_num][cell_num].set_char(ch);
    }

    /// Update the `ch`, `fg`, and `bg` attributes of an indivudual cell
    pub fn update_cell(&mut self, cell_num: uint, row_num: uint, ch: char, fg: Color, bg: Color) {
        self.get_cell_mut(cell_num, row_num).set(ch, fg, bg);
    }

    pub fn get_cell_mut(&mut self, cell_num: uint, row_num: uint) -> &mut Cell {
        &mut self.rows[row_num][cell_num]
    }
}


pub struct Cell {
    pub bg: Color,
    pub fg: Color,
    pub ch: char,
    pub x: uint,
    pub y: uint,
    pub dirty: bool
}


impl Cell {
    pub fn new() -> Cell {
        Cell {
            bg: Color::Default,
            fg: Color::Default,
            ch: ' ',
            x: 0,
            y: 0,
            dirty: true
        }
    }

    pub fn set_char(&mut self, ch: char) {
        if self.ch != ch {
            self.dirty = true;
            self.ch = ch;
        }
    }

    pub fn set(&mut self, ch: char, fg: Color, bg: Color) {
        if self.ch != ch || self.fg != fg || self.bg != bg {
            self.dirty = true;
        }
        self.ch = ch;
        self.fg = fg;
        self.bg = bg;
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


#[cfg(test)]
mod tests {
    use uibuf::UIBuffer;
    use rustbox::Color;

    fn setup_uibuf() -> UIBuffer {
        UIBuffer::new(50, 50)
    }

    #[test]
    fn grid_is_created_correctly() {
        let uibuf = setup_uibuf();
        assert_eq!(uibuf.height, 50);
        assert_eq!(uibuf.width, 50);
        assert_eq!(uibuf.rows.len(), 50);
        assert_eq!(uibuf.rows[0][5].ch, ' ');
    }

    #[test]
    fn fill_updates_contents_of_all_cells() {
        let mut uibuf = setup_uibuf();
        uibuf.fill('x');

        // check some random cells
        assert_eq!(uibuf.rows[20][5].ch, 'x');
        assert_eq!(uibuf.rows[0][30].ch, 'x');
    }

    #[test]
    fn update_cell_content_updates_a_single_cell() {
        let mut uibuf = setup_uibuf();
        // update cell 10 in row 32
        uibuf.update_cell_content(10, 32, 'y');

        assert_eq!(uibuf.rows[32][10].ch, 'y');
    }

    #[test]
    fn update_cell_updates_all_attrs_of_cell() {
        let mut uibuf = setup_uibuf();
        let cell_num = 10u;
        let row_num = 0u;
        let ch = 'q';
        let fg = Color::Red;
        let bg = Color::Blue;

        uibuf.update_cell(cell_num, row_num, ch, fg, bg);

        let cell = &uibuf.rows[row_num][cell_num];
        assert_eq!(cell.ch, ch);

        // FIXME(greg): this fails cos '==' is not implemented for rustbox::Color.
        // assert_eq!(cell.fg, fg);
        // assert_eq!(cell.bg, bg);
    }

}
