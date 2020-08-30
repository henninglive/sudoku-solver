# sudoku-solver
Simple backtracking sudoku solver written in rust.

### Example
```text
$ sudoku-solver "903 020 070
                 160 000 020
                 700 009 300

                 095 008 040
                 006 000 900
                 010 900 630

                 004 300 007
                 080 000 060
                 070 010 208"

┌─────┬─────┬─────┐
│ 943 │ 521 │ 876 │
│ 168 │ 743 │ 529 │
│ 752 │ 869 │ 314 │
├─────┼─────┼─────┤
│ 295 │ 638 │ 741 │
│ 436 │ 172 │ 985 │
│ 817 │ 954 │ 632 │
├─────┼─────┼─────┤
│ 624 │ 385 │ 197 │
│ 581 │ 297 │ 463 │
│ 379 │ 416 │ 258 │
└─────┴─────┴─────┘

```

### Usage
The CLI will read 9x9/81 numbers from the command line arguments and will ignore all other characters.
This means you can format the board anyway you want.
```text
$ sudoku-solver 903 020 070 160 000 020 700 009 300 095 008 040 006 000 900 010 900 630 004 300 007 080 000 060 070 010 208

$ sudoku-solver "9,0,3, 0,2,0, 0,7,0,
                 1,6,0, 0,0,0, 0,2,0,
                 7,0,0, 0,0,9, 3,0,0,

                 0,9,5, 0,0,8, 0,4,0,
                 0,0,6, 0,0,0, 9,0,0,
                 0,1,0, 9,0,0, 6,3,0,

                 0,0,4, 3,0,0, 0,0,7,
                 0,8,0, 0,0,0, 0,6,0,
                 0,7,0, 0,1,0, 2,0,8,"
```


### Build and Run
1. Ensure you have current version of `cargo` and [Rust](https://www.rust-lang.org/) installed
2. Clone the project `$ git clone https://github.com/henninglive/sudoku-solver/ && cd sudoku-solver`
3. Build the project `$ cargo build --release` (NOTE: There is a large performance differnce when compiling without optimizations, so I recommend alwasy using `--release` to enable to them)
4. Once complete, the binary will be located at `target/release/sudoku-solver`
5. Use `$ cargo run --release -- 903020070...` to build and then run, in one step
