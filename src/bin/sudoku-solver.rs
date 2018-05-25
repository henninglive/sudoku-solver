extern crate sudoku_solver as ss;
use ss::DSIZE;

fn main() {
    let data = std::env::args()
        .skip(1)
        .flat_map(|arg| arg.chars().collect::<Vec<_>>())
        .filter_map(|c| c.to_digit(10).map(|i| i as u8))
        .collect::<Vec<_>>();

    let cells = data.get(..(DSIZE * DSIZE))
        .expect("Incomplete board");

    let mut b = ss::Board::new(cells)
        .expect("Invalid board");

    b.solve().expect("Failed to solve board");

    println!("{}", b.text());
}