use std::ops::{BitAnd, BitAndAssign, Not};
use fmt::{Debug, Formatter};
use std::char::from_digit;
use std::fmt::{Write, Display};
use std::fmt;

const SIZE: usize = 3;
const DSIZE: usize = SIZE * SIZE;

/// Sudoku cell
#[derive(Copy, Clone, PartialEq, Eq)]
struct Cell(u16);

/// Sudoku board
#[derive(Clone, PartialEq, Eq)]
pub struct Board(Box<[Cell]>);

/// Iterator yielding all possible candidate for a `Cell` as new `Cell`s.
#[derive(Debug)]
struct Guesses(Cell, usize);

impl Cell {
    const MASK: u16 = (1u16 << DSIZE) - 1;

    /// Constructs a new `Cell` with no candidates.
    const fn none() -> Cell {
        Cell(0)
    }

    /// Constructs a new `Cell` with all values (1..=9) as candidates.
    const fn all() -> Cell {
        Cell(Cell::MASK & <u16>::max_value())
    }

    /// Constructs a new `Cell` from bits with each bit 0..9 representing a possible candidate 1..=9.
    const fn from_bits(bits: u16) -> Cell {
        Cell(Cell::MASK & bits)
    }

    /// Returns an iterator yielding all possible candidate for this cell as new `Cell`s.
    const fn guesses(&self) -> Guesses {
        Guesses(*self, 0)
    }

    /// Returns the number of possible candidate for this cell.
    const fn num_candidates(&self) -> u32 {
        self.0.count_ones()
    }

    /// Constructs a new `Cell` from value (0..=9).
    fn from_value(value: u8) -> Cell {
        match value {
            0 => Cell::all(),
            i if i as usize > DSIZE => panic!("Cell value must be less then {}", DSIZE),
            i => Cell(1 << (i - 1)),
        }
    }

    /// Returns the value for this cell if there is exactly one possible candidate.
    fn value(&self) -> Option<u8> {
        if self.num_candidates() == 1 {
            for i in 0.. {
                if (self.0 & 1 << i) != 0 {
                    return Some(i + 1);
                }
            }
        }
        None
    }

    /// Update possible candidates based on the value a cell.
    fn update_candidates(&mut self, cell: Cell) -> Result<(), ()> {
        if cell.num_candidates() == 1 {
            if !*self & cell != Cell::none() {
                return Err(());
            }
            *self &= !cell;
        }
        Ok(())
    }

    /// Update possible candidates for a cell based on candidates.
    fn update_cell(&mut self, candidates: Cell) -> Result<bool, ()> {
        if self.num_candidates() != 1 {
            let prev = *self;
            *self &= candidates;
            if *self == Cell::none() {
                return Err(());
            }
            Ok(prev != *self)
        } else {
            Ok(false)
        }
    }
}

impl BitAnd for Cell {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Cell(self.0 & rhs.0)
    }
}

impl BitAndAssign for Cell {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for Cell {
    type Output = Cell;
    fn not(self) -> Self::Output {
        Cell(Cell::MASK & !self.0)
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Cell")
            .field(&format!("{:#09b}", self.0))
            .finish()
    }
}

impl Iterator for Guesses {
    type Item = Cell;

    fn next(&mut self) -> Option<Cell> {
        while self.1 < DSIZE {
            if ((self.0).0 & 1 << self.1) != 0 {
                let s = Some(Cell(1 << self.1));
                self.1 += 1;
                return s;
            }
            self.1 += 1;
        }
        None
    }
}

impl Board {
    pub const SIZE: usize = SIZE;
    pub const DSIZE: usize = DSIZE;

    pub fn from_values(data: &[u8]) -> Board {
        if data.len() != DSIZE * DSIZE {
            panic!("Board must have {} cells", DSIZE * DSIZE)
        }

        let cells = data
            .iter()
            .map(|i| Cell::from_value(*i))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Board(cells)
    }

    pub fn from_bits(data: &[u16]) -> Board {
        if data.len() != DSIZE * DSIZE {
            panic!("Board must have {} cells", DSIZE * DSIZE)
        }

        let cells = data
            .iter()
            .map(|i| Cell::from_bits(*i))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Board(cells)
    }

    fn solve_rows(&mut self) -> Result<bool, ()> {
        let mut changed = false;
        for row in self.0.chunks_mut(DSIZE) {
            let mut candidates = Cell::all();

            for cell in row.iter() {
                candidates.update_candidates(*cell)?;
            }

            for cell in row.iter_mut() {
                changed |= cell.update_cell(candidates)?;
            }
        }

        Ok(changed)
    }

    fn solve_columns(&mut self) -> Result<bool, ()> {
        let mut changed = false;
        for x in 0..DSIZE {
            let mut candidates = Cell::all();

            for y in 0..DSIZE {
                let cell = self.0[y * DSIZE + x];
                candidates.update_candidates(cell)?;
            }

            for y in 0..DSIZE {
                let cell = &mut self.0[y * DSIZE + x];
                changed |= cell.update_cell(candidates)?;
            }
        }

        Ok(changed)
    }

    fn solve_squares(&mut self) -> Result<bool, ()> {
        let mut changed = false;
        for sy in 0..SIZE {
            for sx in 0..SIZE {
                let mut candidates = Cell::all();

                for y in 0..SIZE {
                    for x in 0..SIZE {
                        let cell = self.0[(sy * SIZE + y) * DSIZE + sx * SIZE + x];
                        candidates.update_candidates(cell)?;
                    }
                }

                for y in 0..SIZE {
                    for x in 0..SIZE {
                        let cell = &mut self.0[(sy * SIZE + y) * DSIZE + sx * SIZE + x];
                        changed |= cell.update_cell(candidates)?;
                    }
                }
            }
        }

        Ok(changed)
    }

    fn solve_all(&mut self) -> Result<(), ()> {
        while self.solve_rows()? | self.solve_columns()? | self.solve_squares()? {}
        Ok(())
    }

    fn solved(&self) -> bool {
        self.0.iter().all(|cell| cell.num_candidates() == 1)
    }

    pub fn solve(mut self) -> Result<Self, Self> {
        match self.solve_all() {
            Ok(()) => (),
            Err(()) => return Err(self),
        }

        if self.solved() {
            return Ok(self);
        }

        for i in 0..(DSIZE * DSIZE) {
            match self.0[i].num_candidates() {
                0 => unreachable!(),
                1 => (),
                _ => {
                    for guess in self.0[i].guesses() {
                        let mut new = self.clone();
                        new.0[i] = guess;
                        match new.solve() {
                            // We found a solution
                            Ok(board) => return Ok(board),
                            Err(_) => {
                                // We hit an error, we now know that
                                // this is not valid value for this cell.
                                self.0[i] &= !guess;
                            }
                        }
                    }
                }
            }
        }
        Err(self)
    }
}

fn debug_line(f: &mut Formatter<'_>, start: char, line: char, cross: char, alt_cross: char, end: char) -> fmt::Result {
    f.write_char(start)?;
    for i in 0..DSIZE {
        for _ in 0..(SIZE + 2) {
            f.write_char(line)?;
        }

        f.write_char(
            if i == (DSIZE - 1) {
                end
            } else if i % SIZE == SIZE - 1 {
                alt_cross
            } else {
                cross
            }
        )?;
    }
    f.write_char('\n')
}

impl Debug for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char('\n')?;

        debug_line(f, '╔', '═', '╤', '╦', '╗')?;

        for (i, row) in self.0.chunks(DSIZE).enumerate() {
            for cell_y in 0..SIZE {
                for (j, cell) in row.iter().enumerate() {
                    if j == 0 {
                        f.write_str("║ ")?;
                    } else if j % SIZE == 0 {
                        f.write_str(" ║ ")?;
                    } else {
                        f.write_str(" │ ")?;
                    }

                    for cell_x in 0..SIZE {
                        let n = cell_y * SIZE + cell_x;
                        if cell.0 & 1 << n != 0 {
                            f.write_char(from_digit((n + 1) as u32, 10).unwrap())?;
                        } else {
                            f.write_char(' ')?;
                        }
                    }
                }
                f.write_str(" ║\n")?;
            }

            if i == DSIZE - 1 {
                debug_line(f, '╚', '═', '╧', '╩', '╝')?;
            } else if i % SIZE == SIZE - 1 {
                debug_line(f, '╠', '═', '╪', '╬', '╣')?;
            } else {
                debug_line(f, '╟', '─', '┼', '╫', '╢')?;
            }
        }
        Ok(())
    }
}

fn display_line(f: &mut Formatter<'_>, start: char, line: char, cross: char, end: char) -> fmt::Result {
    f.write_char(start)?;
    for i in 0..SIZE {
        for _ in 0..(SIZE + 2) {
            f.write_char(line)?;
        }

        f.write_char(
            if i == (SIZE - 1) {
                end
            } else {
                cross
            }
        )?;
    }
    f.write_char('\n')
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char('\n')?;

        display_line(f, '┌', '─', '┬', '┐')?;

        for (y, row) in self.0.chunks(DSIZE).enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if x == 0 {
                    f.write_str("│ ")?;
                } else if x % SIZE == 0 {
                    f.write_str(" │ ")?;
                }

                f.write_char(match cell.value() {
                    Some(value) => from_digit(value as u32, 10).unwrap(),
                    None => ' ',
                })?;
            }
            f.write_str(" │\n")?;

            if y == DSIZE - 1 {
                display_line(f, '└', '─', '┴', '┘')?;
            } else if y % SIZE == SIZE - 1 {
                display_line(f, '├', '─', '┼', '┤')?;
            }
        }
        Ok(())
    }
}

#[test]
fn test_solve_rows_valid() {
    let mut board = Board::from_values(&[
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        0, 2, 3, 4, 5, 6, 7, 8, 9,
        0, 0, 3, 4, 5, 6, 7, 8, 9,
        0, 0, 0, 4, 5, 6, 7, 8, 9,
        0, 0, 0, 0, 5, 6, 7, 8, 9,
        0, 0, 0, 0, 0, 6, 7, 8, 9,
        0, 0, 0, 0, 0, 0, 7, 8, 9,
        0, 0, 0, 0, 0, 0, 0, 8, 9,
        0, 0, 0, 0, 0, 0, 0, 0, 9,
    ]);

    let expected = Board::from_bits(&[
        0b000000001, 0b000000010, 0b000000100, 0b000001000, 0b000010000, 0b000100000, 0b001000000, 0b010000000, 0b100000000,
        0b000000001, 0b000000010, 0b000000100, 0b000001000, 0b000010000, 0b000100000, 0b001000000, 0b010000000, 0b100000000,
        0b000000011, 0b000000011, 0b000000100, 0b000001000, 0b000010000, 0b000100000, 0b001000000, 0b010000000, 0b100000000,
        0b000000111, 0b000000111, 0b000000111, 0b000001000, 0b000010000, 0b000100000, 0b001000000, 0b010000000, 0b100000000,
        0b000001111, 0b000001111, 0b000001111, 0b000001111, 0b000010000, 0b000100000, 0b001000000, 0b010000000, 0b100000000,
        0b000011111, 0b000011111, 0b000011111, 0b000011111, 0b000011111, 0b000100000, 0b001000000, 0b010000000, 0b100000000,
        0b000111111, 0b000111111, 0b000111111, 0b000111111, 0b000111111, 0b000111111, 0b001000000, 0b010000000, 0b100000000,
        0b001111111, 0b001111111, 0b001111111, 0b001111111, 0b001111111, 0b001111111, 0b001111111, 0b010000000, 0b100000000,
        0b011111111, 0b011111111, 0b011111111, 0b011111111, 0b011111111, 0b011111111, 0b011111111, 0b011111111, 0b100000000,
    ]);

    assert_eq!(board.solve_rows().unwrap(), true);
    assert_eq!(board, expected);
    assert_eq!(board.solve_rows().unwrap(), false);
}

#[test]
fn test_solve_rows_invalid() {
    let mut board = Board::from_values(&[
        1, 1, 1, 1, 1, 1, 1, 1, 1,
        2, 2, 2, 2, 2, 2, 2, 2, 2,
        3, 3, 3, 3, 3, 3, 3, 3, 3,
        4, 4, 4, 4, 4, 4, 4, 4, 4,
        5, 5, 5, 5, 5, 5, 5, 5, 5,
        6, 6, 6, 6, 6, 6, 6, 6, 6,
        7, 7, 7, 7, 7, 7, 7, 7, 7,
        8, 8, 8, 8, 8, 8, 8, 8, 8,
        9, 9, 9, 9, 9, 9, 9, 9, 9,
    ]);

    assert_eq!(board.solve_rows(), Err(()));
}

#[test]
fn test_solve_columns_valid() {
    let mut board = Board::from_values(&[
        1, 0, 0, 0, 0, 0, 0, 0, 0,
        2, 2, 0, 0, 0, 0, 0, 0, 0,
        3, 3, 3, 0, 0, 0, 0, 0, 0,
        4, 4, 4, 4, 0, 0, 0, 0, 0,
        5, 5, 5, 5, 5, 0, 0, 0, 0,
        6, 6, 6, 6, 6, 6, 0, 0, 0,
        7, 7, 7, 7, 7, 7, 7, 0, 0,
        8, 8, 8, 8, 8, 8, 8, 8, 0,
        9, 9, 9, 9, 9, 9, 9, 9, 9,
    ]);

    let expected = Board::from_bits(&[
        0b000000001, 0b000000001, 0b000000011, 0b000000111, 0b000001111, 0b000011111, 0b000111111, 0b001111111, 0b011111111,
        0b000000010, 0b000000010, 0b000000011, 0b000000111, 0b000001111, 0b000011111, 0b000111111, 0b001111111, 0b011111111,
        0b000000100, 0b000000100, 0b000000100, 0b000000111, 0b000001111, 0b000011111, 0b000111111, 0b001111111, 0b011111111,
        0b000001000, 0b000001000, 0b000001000, 0b000001000, 0b000001111, 0b000011111, 0b000111111, 0b001111111, 0b011111111,
        0b000010000, 0b000010000, 0b000010000, 0b000010000, 0b000010000, 0b000011111, 0b000111111, 0b001111111, 0b011111111,
        0b000100000, 0b000100000, 0b000100000, 0b000100000, 0b000100000, 0b000100000, 0b000111111, 0b001111111, 0b011111111,
        0b001000000, 0b001000000, 0b001000000, 0b001000000, 0b001000000, 0b001000000, 0b001000000, 0b001111111, 0b011111111,
        0b010000000, 0b010000000, 0b010000000, 0b010000000, 0b010000000, 0b010000000, 0b010000000, 0b010000000, 0b011111111,
        0b100000000, 0b100000000, 0b100000000, 0b100000000, 0b100000000, 0b100000000, 0b100000000, 0b100000000, 0b100000000,
    ]);

    assert_eq!(board.solve_columns().unwrap(), true);
    assert_eq!(board, expected);
    assert_eq!(board.solve_columns().unwrap(), false);
}

#[test]
fn test_solve_columns_invalid() {
    let mut board = Board::from_values(&[
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        1, 2, 3, 4, 5, 6, 7, 8, 9,
    ]);

    assert_eq!(board.solve_columns(), Err(()));
}

#[test]
fn test_solve_squares_valid() {
    let mut board = Board::from_values(&[
        1, 2, 3, 0, 2, 3, 0, 0, 3,
        4, 5, 6, 4, 5, 6, 4, 5, 6,
        7, 8, 9, 7, 8, 9, 7, 8, 9,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        4, 5, 6, 0, 5, 6, 0, 0, 6,
        7, 8, 9, 7, 8, 9, 7, 8, 9,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        7, 8, 9, 0, 8, 9, 0, 0, 9,
    ]);

    let expected = Board::from_bits(&[
        0b000000001, 0b000000010, 0b000000100, 0b000000001, 0b000000010, 0b000000100, 0b000000011, 0b000000011, 0b000000100,
        0b000001000, 0b000010000, 0b000100000, 0b000001000, 0b000010000, 0b000100000, 0b000001000, 0b000010000, 0b000100000,
        0b001000000, 0b010000000, 0b100000000, 0b001000000, 0b010000000, 0b100000000, 0b001000000, 0b010000000, 0b100000000,
        0b000000111, 0b000000111, 0b000000111, 0b000001111, 0b000001111, 0b000001111, 0b000011111, 0b000011111, 0b000011111,
        0b000001000, 0b000010000, 0b000100000, 0b000001111, 0b000010000, 0b000100000, 0b000011111, 0b000011111, 0b000100000,
        0b001000000, 0b010000000, 0b100000000, 0b001000000, 0b010000000, 0b100000000, 0b001000000, 0b010000000, 0b100000000,
        0b000111111, 0b000111111, 0b000111111, 0b001111111, 0b001111111, 0b001111111, 0b011111111, 0b011111111, 0b011111111,
        0b000111111, 0b000111111, 0b000111111, 0b001111111, 0b001111111, 0b001111111, 0b011111111, 0b011111111, 0b011111111,
        0b001000000, 0b010000000, 0b100000000, 0b001111111, 0b010000000, 0b100000000, 0b011111111, 0b011111111, 0b100000000,
    ]);

    assert_eq!(board.solve_squares().unwrap(), true);
    assert_eq!(board, expected);
    assert_eq!(board.solve_squares().unwrap(), false);
}

#[test]
fn test_solve_squares_invalid() {
    let mut board = Board::from_values(&[
        1, 1, 1, 2, 2, 2, 3, 3, 3,
        1, 1, 1, 2, 2, 2, 3, 3, 3,
        1, 1, 1, 2, 2, 2, 3, 3, 3,
        4, 4, 4, 5, 5, 5, 6, 6, 6,
        4, 4, 4, 5, 5, 5, 6, 6, 6,
        4, 4, 4, 5, 5, 5, 6, 6, 6,
        7, 7, 7, 8, 8, 8, 9, 9, 9,
        7, 7, 7, 8, 8, 8, 9, 9, 9,
        7, 7, 7, 8, 8, 8, 9, 9, 9,
    ]);

    assert_eq!(board.solve_squares(), Err(()));
}

#[test]
fn test_solved() {
    let mut board = Board::from_values(&[
        5, 3, 4, 6, 7, 8, 9, 1, 2,
        6, 7, 2, 1, 9, 5, 3, 4, 8,
        1, 9, 8, 3, 4, 2, 5, 6, 7,
        8, 5, 9, 7, 6, 1, 4, 2, 3,
        4, 2, 6, 8, 5, 3, 7, 9, 1,
        7, 1, 3, 9, 2, 4, 8, 5, 6,
        9, 6, 1, 5, 3, 7, 2, 8, 4,
        2, 8, 7, 4, 1, 9, 6, 3, 5,
        3, 4, 5, 2, 8, 6, 1, 7, 9,
    ]);

    assert_eq!(board.solve_rows().unwrap(), false);
    assert_eq!(board.solve_columns().unwrap(), false);
    assert_eq!(board.solve_squares().unwrap(), false);
    assert!(board.solved());
}

#[test]
fn test_guesses() {
    assert_eq!(
        Cell::all().guesses().collect::<Vec<_>>(),
        (1..=9).map(|i| Cell::from_value(i)).collect::<Vec<_>>()
    );

    assert_eq!(
        Cell::none().guesses().collect::<Vec<_>>(),
        vec![]
    );

    assert_eq!(
        Cell::from_bits(0b11).guesses().collect::<Vec<_>>(),
        vec![Cell::from_value(1), Cell::from_value(2)]
    );
}

#[test]
fn test_board_simple() {
    Board::from_values(&[
        0, 8, 7, 0, 1, 0, 0, 0, 0,
        0, 0, 4, 8, 0, 0, 1, 2, 0,
        0, 0, 1, 7, 0, 5, 6, 0, 9,
        8, 1, 0, 0, 0, 0, 2, 0, 0,
        0, 6, 0, 0, 0, 0, 0, 5, 0,
        0, 0, 9, 0, 0, 0, 0, 6, 4,
        5, 0, 6, 1, 0, 7, 9, 0, 0,
        0, 3, 2, 0, 0, 9, 5, 0, 0,
        0, 0, 0, 0, 6, 0, 4, 7, 0,
    ]).solve().unwrap();
}

#[test]
fn test_board_easy() {
    Board::from_values(&[
        1, 0, 4, 0, 0, 0, 3, 0, 6,
        8, 0, 9, 0, 3, 0, 5, 7, 0,
        0, 0, 0, 0, 7, 0, 1, 0, 0,
        4, 2, 6, 0, 0, 0, 0, 0, 3,
        0, 8, 7, 0, 0, 6, 0, 1, 2,
        3, 0, 0, 0, 0, 0, 0, 0, 9,
        2, 4, 1, 9, 0, 0, 0, 3, 0,
        0, 0, 0, 2, 0, 0, 0, 8, 0,
        7, 0, 0, 5, 0, 3, 0, 0, 0,
    ]).solve().unwrap();
}

#[test]
fn test_board_hard() {
    Board::from_values(&[
        2, 9, 0, 1, 0, 0, 0, 0, 5,
        0, 7, 0, 0, 5, 0, 0, 0, 0,
        0, 8, 0, 0, 0, 0, 6, 0, 0,
        4, 0, 0, 0, 3, 2, 0, 0, 0,
        0, 0, 5, 8, 0, 7, 2, 0, 0,
        0, 0, 0, 9, 6, 0, 0, 0, 1,
        0, 0, 9, 0, 0, 0, 0, 1, 0,
        0, 0, 0, 0, 2, 0, 0, 5, 0,
        6, 0, 0, 0, 0, 1, 0, 7, 2,
    ]).solve().unwrap();
}

#[test]
fn test_board_hard2() {
    Board::from_values(&[
        8, 0, 0, 5, 9, 0, 3, 0, 1,
        0, 2, 0, 7, 0, 0, 8, 0, 0,
        0, 0, 0, 8, 0, 0, 0, 0, 2,
        0, 0, 8, 0, 0, 0, 0, 1, 0,
        0, 0, 0, 3, 0, 5, 0, 0, 0,
        0, 4, 0, 0, 0, 0, 5, 0, 0,
        1, 0, 0, 0, 0, 3, 0, 0, 0,
        0, 0, 7, 0, 0, 4, 0, 8, 0,
        5, 0, 9, 0, 7, 8, 0, 0, 6,
    ]).solve().unwrap();
}

#[test]
fn test_board_evil() {
    Board::from_values(&[
        0, 9, 0, 0, 0, 0, 7, 0, 0,
        0, 0, 0, 0, 1, 0, 0, 0, 8,
        0, 2, 0, 6, 0, 9, 0, 0, 0,
        5, 0, 0, 0, 6, 0, 3, 2, 0,
        3, 0, 0, 9, 0, 2, 0, 0, 5,
        0, 6, 2, 0, 3, 0, 0, 0, 4,
        0, 0, 0, 3, 0, 7, 0, 5, 0,
        9, 0, 0, 0, 4, 0, 0, 0, 0,
        0, 0, 6, 0, 0, 0, 0, 4, 0,
    ]).solve().unwrap();
}


#[test]
fn test_board_evil2() {
    Board::from_values(&[
        2, 0, 0, 0, 8, 5, 0, 9, 1,
        0, 0, 0, 2, 0, 0, 0, 7, 0,
        0, 0, 6, 0, 0, 0, 0, 0, 5,
        6, 0, 0, 0, 0, 9, 0, 0, 0,
        0, 9, 3, 0, 0, 0, 1, 4, 0,
        0, 0, 0, 4, 0, 0, 0, 0, 2,
        4, 0, 0, 0, 0, 0, 8, 0, 0,
        0, 1, 0, 0, 0, 8, 0, 0, 0,
        8, 2, 0, 3, 1, 0, 0, 0, 4,
    ]).solve().unwrap();
}

#[test]
fn test_board_erica() {
    Board::from_values(&[
        9, 0, 3, 0, 2, 0, 0, 7, 0,
        0, 6, 0, 0, 0, 0, 0, 2, 0,
        7, 0, 0, 0, 0, 9, 3, 0, 0,
        0, 9, 5, 0, 0, 8, 0, 4, 0,
        0, 0, 6, 0, 0, 0, 9, 0, 0,
        0, 1, 0, 9, 0, 0, 6, 3, 0,
        0, 0, 4, 3, 0, 0, 0, 0, 7,
        0, 8, 0, 0, 0, 0, 0, 6, 0,
        0, 7, 0, 0, 1, 0, 2, 0, 8,
    ]).solve().unwrap();
}

#[test]
fn test_board_test() {
    Board::from_values(&[
        9, 0, 3, 0, 2, 0, 0, 7, 0,
        1, 6, 0, 0, 0, 0, 0, 2, 0,
        7, 0, 0, 0, 0, 9, 3, 0, 0,
        0, 9, 5, 0, 0, 8, 0, 4, 0,
        0, 0, 6, 0, 0, 0, 9, 0, 0,
        0, 1, 0, 9, 0, 0, 6, 3, 0,
        0, 0, 4, 3, 0, 0, 0, 0, 7,
        0, 8, 0, 0, 0, 0, 0, 6, 0,
        0, 7, 0, 0, 1, 0, 2, 0, 8,
    ]).solve().unwrap();
}

#[test]
fn test_board_empty() {
    Board::from_values(&[
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]).solve().unwrap();
}

#[test]
fn test_board_unsolvable() {
    let board = Board::from_values(&[
        1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1,
    ]);
    assert_eq!(board.clone().solve(), Err(board));
}
