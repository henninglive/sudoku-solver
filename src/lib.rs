pub const SIZE: usize = 3;
pub const DSIZE: usize = SIZE * SIZE;

/// Used bits of Cell::Unknown.
const MASK: u16 = ((1usize << DSIZE) - 1) as u16;

/// Sudoku cell
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Cell {
    /// We know the exact value of the cell so we store that value.
    Known(u8),
    /// There are multiple candidate for this cell. As an optimization
    /// we use the u16 as bitset and set one bit for each possible value.
    Unknown(u16),
}

/// Search status
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    /// Some cells where resolved, continue trying to resolve cells.
    Progressing,
    /// We are unable to progress, we make a guess.
    Halted,
    /// Found a solution.
    Solved,
}

/// Sudoku board
#[derive(Debug)]
pub struct Board(Vec<Cell>);

impl Cell {
    /// Convert cell to bitset, one bit per possible value for cell.
    fn to_bit_vec(&self) -> Vec<bool> {
        match *self {
            Cell::Known( ref value ) => {
                (1..(DSIZE + 1)).map(|i| {
                    if i == *value as usize {
                        true
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>()
            },
            Cell::Unknown( ref cands ) => {
                let mut c = *cands;
                let mut v = Vec::new();
                for _ in 0..DSIZE {
                    if c & 1 == 1 {
                        v.push(true);
                    } else {
                        v.push(false);
                    }
                    c >>= 1;
                }
                v
            },
        }
    }
}

impl Board {
    pub fn new(data: &[u8]) -> Result<Board, ()> {
        if data.len() != DSIZE * DSIZE {
            return Err(());
        }

        let mut b = Board(data.iter().map(|i| {
            match *i {
                0 => Cell::Unknown(MASK & <u16>::max_value()),
                i if i as usize > DSIZE => panic!("Cell value must be less then {}", DSIZE),
                i => Cell::Known(i),
            }
        }).collect::<Vec<_>>());

        b.solve_squares()?;
        b.solve_rows()?;
        b.solve_columns()?;

        Ok(b)
    }

    fn solve_squares(&mut self) -> Result<(), ()> {
        for sy in 0..SIZE {
            for sx in 0..SIZE {
                let mut cand = MASK & <u16>::max_value();
                for y in 0..SIZE {
                    for x in 0..SIZE {
                        let i = (sy * SIZE + y) * DSIZE + sx * SIZE + x;
                        match self.0[i] {
                            Cell::Known(ref value) => {
                                if cand & (1 << (*value - 1)) == 0 {
                                    // Error encountered, we need to backtrack.
                                    return Err(());
                                }
                                cand &= !(1 << (*value - 1))
                            },
                            _ => (),
                        }
                    }
                }

                for y in 0..SIZE {
                    for x in 0..SIZE {
                        let i = (sy * SIZE + y) * DSIZE + sx * SIZE + x;
                        match self.0[i] {
                            Cell::Unknown(ref mut c) => {
                                *c &= cand;
                                if *c == 0 {
                                    // No valid value, we need to backtrack.
                                    return Err(());
                                }
                            },
                            _ => (),
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn solve_rows(&mut self) -> Result<(), ()> {
        for row in self.0.chunks_mut(DSIZE) {
            let mut cand = MASK & <u16>::max_value();
            for i in row.iter() {
                match i {
                    &Cell::Known(ref value) => {
                        if cand & (1 << (*value - 1)) == 0 {
                            // Error encountered, we need to backtrack.
                            return Err(());
                        }
                        cand &= !(1 << (*value - 1))
                    },
                    _ => (),
                }
            }

            for i in row.iter_mut() {
                match i {
                    &mut Cell::Unknown(ref mut c) => {
                        *c &= cand;
                        if *c == 0 {
                            // No valid value, we need to backtrack. 
                            return Err(());
                        }
                    },
                    _ => (),
                }
            }
        }
        Ok(())
    }

    fn solve_columns(&mut self) -> Result<(), ()> {
        for x in 0..DSIZE {
            let mut cand = MASK & <u16>::max_value();
            for y in 0..DSIZE {
                match self.0[y * DSIZE + x] {
                    Cell::Known(ref value) => {
                        if cand & (1 << (*value - 1)) == 0 {
                            // Error encountered, we need to backtrack.
                            return Err(());
                        }
                        cand &= !(1 << (*value - 1))
                    },
                    _ => (),
                }
            }

            for y in 0..DSIZE {
                match self.0[y * DSIZE + x] {
                    Cell::Unknown(ref mut c) => {
                        *c &= cand;
                        if *c == 0 {
                            return Err(());
                        }
                    },
                    _ => (),
                }
            }
        }
        Ok(())
    }

    fn resolve(&mut self) -> Status {
        let mut progressing = false;
        let mut done = true;

        for i in self.0.iter_mut() {
            let new_value = match i {
                &mut Cell::Unknown( ref c ) => {
                    done = false;
                    let mut cand = MASK & *c;
                    match cand.count_ones() {
                        0 => panic!("No valid value"),
                        1 => {
                            let mut i = 0;
                            loop {
                                debug_assert!(i < 16);
                                if 1 & cand == 1 {
                                    break i;
                                }
                                cand >>= 1;
                                i += 1;
                            }
                        },
                        _ => continue,
                    }
                },
                _ => continue,
            };

            *i = Cell::Known( new_value + 1 );
            progressing = true;
        }

        match (done, progressing) {
            (true, _) => Status::Solved,
            (false, true) => Status::Progressing,
            (false, false) => Status::Halted,
        }
    }

    fn guess(&mut self) -> Result<(), ()> {
        for i in 0..(DSIZE * DSIZE) {
            match self.0[i] {
                Cell::Unknown(cand) => {
                    for n in 0..DSIZE {
                        if cand & (1 << n) != 0 {
                            let mut new = Board(self.0.clone());
                            new.0[i] = Cell::Known((n + 1) as u8);

                            return match new.solve() {
                                Ok(()) => {
                                    // We found the solution
                                    self.0 = new.0;
                                    Ok(())
                                },
                                Err(()) => {
                                    // We hit an error, We now know that
                                    // this is not valid value for this cell.
                                    match self.0.get_mut(i).unwrap() {
                                        &mut Cell::Unknown(ref mut cand) => {
                                            *cand &= !(1 << n);
                                            Err(())
                                        },
                                        _ => unreachable!("Unexpected unknown"),
                                    }
                                },
                            };
                        }
                    }
                },
                _ => (),
            }
        }
        unreachable!("No valid guess")
    }

    pub fn solve(&mut self) -> Result<(), ()> {
        loop {
            self.solve_columns()?;
            self.solve_rows()?;
            self.solve_squares()?;
            match self.resolve() {
                Status::Progressing => (),
                Status::Solved => break Ok(()),
                Status::Halted => {
                    if let Ok(_) = self.guess() {
                        break Ok(());
                    }
                },
            }
        }
    }

    pub fn text(&self) -> String {
        let mut s = String::new();
        text_line(&mut s, '╔', '═', '╤', '╦', '╗');

        let board = self.0.iter()
            .map(|cell| cell.to_bit_vec())
            .collect::<Vec<_>>();

        for (i, row) in board.chunks(DSIZE).enumerate() {
            for cell_y in 0..SIZE {
                for (j, cell) in row.iter().enumerate() {
                    if j == 0 {
                        s.push_str("║ ");
                    } else if j % SIZE == 0 {
                        s.push_str(" ║ ");
                    } else {
                        s.push_str(" │ ");
                    }

                    for cell_x in 0..SIZE {
                        let n = cell_y * SIZE + cell_x;
                        if cell[n] {
                            let c = std::char::from_digit((n + 1) as u32, 10)
                                .expect("Printing only support for grids smaller then 3x3.");
                            s.push(c);
                        } else {
                            s.push(' ');
                        }
                    }
                }
                s.push_str(" ║\n");
            }

            if i == DSIZE - 1 {
                text_line(&mut s, '╚', '═', '╧', '╩', '╝');
            } else if i % SIZE == SIZE - 1 {
                text_line(&mut s, '╠', '═', '╪', '╬', '╣');
            } else {
                text_line(&mut s, '╟', '─', '┼', '╫', '╢');
            }
        }
        s
    }
}

fn text_line(s: &mut String, start: char, line: char,
    cross: char, alt_cross: char, end: char) {
    s.push(start);

    for i in 0..DSIZE {
        for _ in 0..(SIZE+2) {
            s.push(line);
        }

        s.push(
            if i == (DSIZE - 1) {
                end
            } else if i % SIZE == SIZE - 1{
                alt_cross
            } else {
                cross
            }
        );
    }
    s.push('\n');
}

#[cfg(test)]
mod tests {
    use ::*;

    fn test_board(data: &[u8]) -> Board {
        Board(
            data.iter()
            .map(|i| {
                match *i {
                    0 => Cell::Unknown(MASK & <u16>::max_value()),
                    i if i as usize > DSIZE => panic!("Cell value must be less then {}", DSIZE),
                    i => Cell::Known(i),
                }
            })
            .collect::<Vec<_>>()
        )
    }

    #[test]
    fn validate_square() {
        let mut valid = test_board(&[
            1,2,3,  0,2,3,  0,0,0,
            4,5,6,  4,5,6,  4,5,6,
            7,8,9,  7,8,9,  7,8,9,

            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,

            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
        ]);

        assert!(valid.solve_squares().is_ok());
        assert_eq!(valid.0[3], Cell::Unknown(0b1));
        assert_eq!(valid.0[6], Cell::Unknown(0b111));
        assert_eq!(valid.0[7], Cell::Unknown(0b111));
        assert_eq!(valid.0[8], Cell::Unknown(0b111));

        let mut invalid = test_board(&[
            1,1,1,  0,0,0,  0,0,0,
            2,2,2,  0,0,0,  0,0,0,
            3,3,3,  0,0,0,  0,0,0,

            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,

            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
        ]);

        assert!(invalid.solve_squares().is_err());
    }

    #[test]
    fn validate_row() {
        let mut valid = test_board(&[
            1,2,3,  4,5,6,  7,8,9,
            0,2,3,  4,5,6,  7,8,9,
            0,0,0,  4,5,6,  7,8,9,

            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,

            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
        ]);

        assert!(valid.solve_rows().is_ok());
        assert_eq!(valid.0[DSIZE], Cell::Unknown(0b1));
        assert_eq!(valid.0[2 * DSIZE],     Cell::Unknown(0b111));
        assert_eq!(valid.0[2 * DSIZE + 1], Cell::Unknown(0b111));
        assert_eq!(valid.0[2 * DSIZE + 2], Cell::Unknown(0b111));

        let mut invalid = test_board(&[
            1,1,1,  2,2,2,  3,3,3,
            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,

            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,

            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
            0,0,0,  0,0,0,  0,0,0,
        ]);

        assert!(invalid.solve_rows().is_err());
    }

    #[test]
    fn validate_columns() {
        let mut valid = test_board(&[
            1,0,0,  0,0,0,  0,0,0,
            2,2,0,  0,0,0,  0,0,0,
            3,3,0,  0,0,0,  0,0,0,

            4,4,4,  0,0,0,  0,0,0,
            5,5,5,  0,0,0,  0,0,0,
            6,6,6,  0,0,0,  0,0,0,

            7,7,7,  0,0,0,  0,0,0,
            8,8,8,  0,0,0,  0,0,0,
            9,9,9,  0,0,0,  0,0,0,
        ]);

        assert!(valid.solve_columns().is_ok());
        assert_eq!(valid.0[1], Cell::Unknown(1));
        assert_eq!(valid.0[2],             Cell::Unknown(0b111));
        assert_eq!(valid.0[DSIZE + 2],     Cell::Unknown(0b111));
        assert_eq!(valid.0[2 * DSIZE + 2], Cell::Unknown(0b111));

        let mut invalid = test_board(&[
            1,0,0,  0,0,0,  0,0,0,
            1,0,0,  0,0,0,  0,0,0,
            1,0,0,  0,0,0,  0,0,0,

            2,0,0,  0,0,0,  0,0,0,
            2,0,0,  0,0,0,  0,0,0,
            2,0,0,  0,0,0,  0,0,0,

            3,0,0,  0,0,0,  0,0,0,
            3,0,0,  0,0,0,  0,0,0,
            4,0,0,  0,0,0,  0,0,0,
        ]);

        assert!(invalid.solve_columns().is_err());
    }

    fn solve_board(board: &[u8]) {
        let mut board = Board::new(&board).unwrap();
        assert!(board.solve().is_ok());
        for e in board.0.iter() {
            match e {
                &Cell::Known{ .. } => (),
                _ => panic!("Unknown cell value after solve"),
            }
        }
    }

    #[test]
    fn board_simple() {
        solve_board(&[
            0,8,7,  0,1,0,  0,0,0,
            0,0,4,  8,0,0,  1,2,0,
            0,0,1,  7,0,5,  6,0,9,

            8,1,0,  0,0,0,  2,0,0,
            0,6,0,  0,0,0,  0,5,0,
            0,0,9,  0,0,0,  0,6,4,

            5,0,6,  1,0,7,  9,0,0,
            0,3,2,  0,0,9,  5,0,0,
            0,0,0,  0,6,0,  4,7,0,
        ][..]);
    }

    #[test]
    fn board_easy() {
        solve_board(&[
            1,0,4,  0,0,0,  3,0,6,
            8,0,9,  0,3,0,  5,7,0,
            0,0,0,  0,7,0,  1,0,0,

            4,2,6,  0,0,0,  0,0,3,
            0,8,7,  0,0,6,  0,1,2,
            3,0,0,  0,0,0,  0,0,9,

            2,4,1,  9,0,0,  0,3,0,
            0,0,0,  2,0,0,  0,8,0,
            7,0,0,  5,0,3,  0,0,0,
        ][..]);
    }

    #[test]
    fn board_hard() {
        solve_board(&[
            2,9,0,  1,0,0,  0,0,5,
            0,7,0,  0,5,0,  0,0,0,
            0,8,0,  0,0,0,  6,0,0,

            4,0,0,  0,3,2,  0,0,0,
            0,0,5,  8,0,7,  2,0,0,
            0,0,0,  9,6,0,  0,0,1,

            0,0,9,  0,0,0,  0,1,0,
            0,0,0,  0,2,0,  0,5,0,
            6,0,0,  0,0,1,  0,7,2,
        ][..]);
    }

    #[test]
    fn board_hard2() {
        solve_board(&[
            8,0,0, 5,9,0, 3,0,1,
            0,2,0, 7,0,0, 8,0,0,
            0,0,0, 8,0,0, 0,0,2,

            0,0,8, 0,0,0, 0,1,0,
            0,0,0, 3,0,5, 0,0,0,
            0,4,0, 0,0,0, 5,0,0,

            1,0,0, 0,0,3, 0,0,0,
            0,0,7, 0,0,4, 0,8,0,
            5,0,9, 0,7,8, 0,0,6,
        ][..]);
    }

    #[test]
    fn board_evil() {
        solve_board(&[
            0,9,0,  0,0,0,  7,0,0,
            0,0,0,  0,1,0,  0,0,8,
            0,2,0,  6,0,9,  0,0,0,

            5,0,0,  0,6,0,  3,2,0,
            3,0,0,  9,0,2,  0,0,5,
            0,6,2,  0,3,0,  0,0,4,

            0,0,0,  3,0,7,  0,5,0,
            9,0,0,  0,4,0,  0,0,0,
            0,0,6,  0,0,0,  0,4,0,
        ][..]);
    }

    #[test]
    fn board_evil2() {
        solve_board(&[
            2,0,0, 0,8,5, 0,9,1,
            0,0,0, 2,0,0, 0,7,0,
            0,0,6, 0,0,0, 0,0,5,

            6,0,0, 0,0,9, 0,0,0,
            0,9,3, 0,0,0, 1,4,0,
            0,0,0, 4,0,0, 0,0,2,

            4,0,0, 0,0,0, 8,0,0,
            0,1,0, 0,0,8, 0,0,0,
            8,2,0, 3,1,0, 0,0,4,
        ][..]);
    }

    #[test]
    fn board_erica() {
        solve_board(&[
            9,0,3,  0,2,0,  0,7,0,
            0,6,0,  0,0,0,  0,2,0,
            7,0,0,  0,0,9,  3,0,0,

            0,9,5,  0,0,8,  0,4,0,
            0,0,6,  0,0,0,  9,0,0,
            0,1,0,  9,0,0,  6,3,0,

            0,0,4,  3,0,0,  0,0,7,
            0,8,0,  0,0,0,  0,6,0,
            0,7,0,  0,1,0,  2,0,8,
        ][..]);
    }

    #[test]
    fn board_test() {
        solve_board(&[
            9,0,3,  0,2,0,  0,7,0,
            1,6,0,  0,0,0,  0,2,0,
            7,0,0,  0,0,9,  3,0,0,

            0,9,5,  0,0,8,  0,4,0,
            0,0,6,  0,0,0,  9,0,0,
            0,1,0,  9,0,0,  6,3,0,

            0,0,4,  3,0,0,  0,0,7,
            0,8,0,  0,0,0,  0,6,0,
            0,7,0,  0,1,0,  2,0,8,
        ][..]);
    }
}
