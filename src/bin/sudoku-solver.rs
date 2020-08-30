extern crate sudoku_solver as ss;

use ss::Board;

fn main() {
    let data = std::env::args()
        .skip(1)
        .flat_map(|arg| arg.chars().collect::<Vec<_>>())
        .filter_map(|c| c.to_digit(10).map(|i| i as u8))
        .collect::<Vec<_>>();

    let cells = data.get(..(Board::DSIZE * Board::DSIZE))
        .expect("Incomplete board");

    let board = Board::from_values(cells)
        .solve()
        .expect("Failed to solve board");

    println!("{}", board);
}