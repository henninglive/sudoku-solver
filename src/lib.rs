const SIZE: usize = 3;
const DSIZE: usize = SIZE * SIZE;
const MASK: u16 = ((1usize << DSIZE) - 1) as u16;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Cell {
    Known{ value: u8 },
    Unknown{ candidates: u16 },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    Progressing,
    Halted,
    Solved,
}

#[derive(Debug)]
pub struct Grid(Vec<Cell>);

impl Cell {
    fn to_bit_vec(&self) -> Vec<bool> {
        match *self {
            Cell::Known{ ref value } => {
                (1..(DSIZE + 1)).map(|i| {
                    if i == *value as usize {
                        true
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>()
            },
            Cell::Unknown{ ref candidates } => {
                let mut c = *candidates;
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

impl Grid {
    pub fn new(data: &[u8]) -> Grid {
        assert!(data.len() == DSIZE * DSIZE);
        assert!((1usize).checked_shl((DSIZE - 1) as u32).unwrap() < <u16>::max_value() as usize);

        Grid(data.iter().map(|i| {
            match *i {
                0 => Cell::Unknown{ candidates: <u16>::max_value() },
                i if i as usize > DSIZE => panic!(),
                i => Cell::Known{ value: i},
            }
        }).collect::<Vec<_>>())
    }

    fn solve_squares(&mut self) {
        for sy in 0..SIZE {
            for sx in 0..SIZE {
                #[cfg(debug_assertions)]
                let mut test = [false; DSIZE];

                let mut cand = <u16>::max_value();
                for y in 0..SIZE {
                    for x in 0..SIZE {
                        let i = (sy * SIZE + y) * DSIZE + sx * SIZE + x;
                        match self.0[i] {
                            Cell::Known{ref value} => {
                                #[cfg(debug_assertions)]
                                {
                                    assert_eq!(test[(value - 1) as usize], false);
                                    test[(value - 1) as usize] = true;
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
                            Cell::Unknown{ref mut candidates} => *candidates &= cand,
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    fn solve_rows(&mut self) {
        for row in self.0.chunks_mut(DSIZE) {
            #[cfg(debug_assertions)]
            let mut test = [false; DSIZE];

            let mut cand = <u16>::max_value();
            for i in row.iter() {
                match i {
                    &Cell::Known{ref value} => {
                        #[cfg(debug_assertions)]
                        {
                            assert_eq!(test[(value - 1) as usize], false);
                            test[(value - 1) as usize] = true;
                        }
                        cand &= !(1 << (*value - 1))
                    },
                    _ => (),
                }
            }

            for i in row.iter_mut() {
                match i {
                    &mut Cell::Unknown{ref mut candidates} => *candidates &= cand,
                    _ => (),
                }
            }
        }
    }

    fn solve_columns(&mut self) {
        for x in 0..DSIZE {
            #[cfg(debug_assertions)]
            let mut test = [false; DSIZE];

            let mut cand = <u16>::max_value();
            for y in 0..DSIZE {
                match self.0[y * DSIZE + x] {
                    Cell::Known{ref value} => {
                        #[cfg(debug_assertions)]
                        {
                            assert_eq!(test[(value - 1) as usize], false);
                            test[(value - 1) as usize] = true;
                        }
                        cand &= !(1 << (*value - 1))
                    },
                    _ => (),
                }
            }

            for y in 0..DSIZE {
                match self.0[y * DSIZE + x] {
                    Cell::Unknown{ref mut candidates} => *candidates &= cand,
                    _ => (),
                }
            }
        }
    }

    fn resolve(&mut self) -> Status {
        let mut progressing = false;
        let mut done = true;

        for i in self.0.iter_mut() {
            let new_value = match i {
                &mut Cell::Unknown{ ref mut candidates } => {
                    done = false;
                    let mut cand = MASK & *candidates;
                    match cand.count_ones() {
                        0 => panic!("No valid candidates"),
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

            *i = Cell::Known{ value: new_value + 1 };
            progressing = true;
        }

        match (done, progressing) {
            (true, _) => Status::Solved,
            (false, true) => Status::Progressing,
            (false, false) => Status::Halted,
        }
    }

    pub fn solve(&mut self) {
        loop {
            self.solve_columns();
            self.solve_rows();
            self.solve_squares();
            match self.resolve() {
                Status::Solved => break,
                Status::Progressing => (),
                Status::Halted => {
                    for (i, cell) in self.0.iter().enumerate() {
                        match cell {
                            &Cell::Unknown{ ref candidates } => {
                                let mut cand = *candidates;
                                let mut n = 1u8;
                                while cand & 1 == 0 && n <= DSIZE as u8 {
                                    cand >>= 1;
                                    n += 1;
                                }

                                let mut new = Grid(self.0.clone());
                                new.0[i] = Cell::Known{value: n};
                                new.solve();
                                return;
                            },
                            _ => (),
                        }
                    }
                },
            }
        }
    }

    pub fn text(&self) -> String {
        let mut s = String::new();
        text_line(&mut s, '╔', '═', '╤', '╦', '╗');

        let grid = self.0.iter()
            .map(|cell| cell.to_bit_vec())
            .collect::<Vec<_>>();

        for (i, row) in grid.chunks(DSIZE).enumerate() {
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

    fn test_board(board: &[u8]) {
        let mut grid = Grid::new(&board);
        grid.solve();
        for e in grid.0.iter() {
            match e {
                &Cell::Known{ .. } => (),
                _ => panic!("Unknown cell value after solve"),
            }
        }
    }

    #[test]
    fn board_simple() {
        test_board(&[
            0, 8, 7,  0, 1, 0,  0, 0, 0,
            0, 0, 4,  8, 0, 0,  1, 2, 0,
            0, 0, 1,  7, 0, 5,  6, 0, 9,

            8, 1, 0,  0, 0, 0,  2, 0, 0,
            0, 6, 0,  0, 0, 0,  0, 5, 0,
            0, 0, 9,  0, 0, 0,  0, 6, 4,

            5, 0, 6,  1, 0, 7,  9, 0, 0,
            0, 3, 2,  0, 0, 9,  5, 0, 0,
            0, 0, 0,  0, 6, 0,  4, 7, 0,
        ][..]);
    }

    #[test]
    fn board_easy() {
        test_board(&[
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
        test_board(&[
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
        test_board(&[
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
        test_board(&[
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
        test_board(&[
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
        test_board(&[
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
        test_board(&[
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