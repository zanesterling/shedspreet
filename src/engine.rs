pub struct Spreadsheet {
    // The maximum X and Y values of filled cells in the sheet.
    max_x: usize,
    max_y: usize,
    // The width and height of the cells array.
    arr_w: usize,
    arr_h: usize,

    cells: Vec<Cell>,
}

impl Spreadsheet {
    pub fn show_cell(&self, x: usize, y: usize) -> String {
        let mut cell = &empty_cell();
        if x < self.arr_w && y < self.arr_h {
            cell = &self.cells[x + y * self.arr_w]
        };
        cell.show()
    }

    pub fn get_max_dims(&self) -> (usize, usize) {
        (self.max_x, self.max_y)
    }

    // Extends the internal array if necessary and sets cell (x,y) to contents.
    pub fn set(&mut self, x: usize, y: usize, contents: String) {
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }

        if x >= self.arr_w || y >= self.arr_h {
            self.grow_array_to_fit(x, y)
        }

        let new_cell = &mut self.cells[x + y * self.arr_w];
        new_cell.contents = contents;
    }

    fn grow_array_to_fit(&mut self, x: usize, y: usize) {
        let mut new_arr_w = self.arr_w;
        let mut new_arr_h = self.arr_h;
        while new_arr_w <= x {
            new_arr_w *= 2;
        }

        while new_arr_h <= y {
            new_arr_h *= 2;
        }
        let mut new_cells = Vec::with_capacity(new_arr_w * new_arr_h);
        for yy in 0..self.arr_h {
            for xx in 0..self.arr_w {
                new_cells.push(self.cells[xx + yy * self.arr_w].clone());
            }
            for _ in self.arr_w..new_arr_w {
                new_cells.push(empty_cell());
            }
        }
        new_cells.resize(new_arr_w * new_arr_h, empty_cell());
        self.cells = new_cells;
        self.arr_w = new_arr_w;
        self.arr_h = new_arr_h;
    }
}

pub fn new_sheet() -> Spreadsheet {
    Spreadsheet {
        max_x: 0,
        max_y: 0,
        arr_w: 1,
        arr_h: 1,
        cells: vec![empty_cell()],
    }
}

#[derive(Clone)]
pub struct Cell {
    contents: String,
    backrefs: Vec<CellRef>,
}

impl Cell {
    pub fn show(&self) -> String {
        // TODO: Compute from formula.
        // OPTIMIZE: Don't clone on every show.
        self.contents.clone()
    }
}

#[derive(Clone)]
struct CellRef {
    x: usize,
    y: usize,
}

impl CellRef {
    fn of(x: usize, y: usize) -> CellRef {
        CellRef { x: x, y: y }
    }
}

pub fn empty_cell() -> Cell {
    Cell {
        contents: "".to_string(),
        backrefs: vec![],
    }
}
