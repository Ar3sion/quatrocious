use crate::{Board, Solution, Solver};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io;
use std::io::Write;
use std::ops::Div;
use std::path::Path;
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct BoardUpToSymmetry(Board);

impl BoardUpToSymmetry {
    fn new(board: Board) -> Self {
        let symmetric = board.symmetric_board();
        Self(board.min(symmetric))
    }
}

fn generate_recursively(board: Board, move_count: u32, result: &mut BTreeSet<BoardUpToSymmetry>) {
    if board.filled_squares() == move_count {
        result.insert(BoardUpToSymmetry::new(board));
    } else {
        for column in 0..Board::WIDTH {
            if let Ok(new_board) = board.make_move(column) {
                if !result.contains(&BoardUpToSymmetry::new(board)) {
                    generate_recursively(new_board, move_count, result);
                }
            }
        }
    }
}

fn generate_opening_boards(move_count: u32) -> Vec<BoardUpToSymmetry> {
    //Removes symmetric boards
    let mut result = BTreeSet::new();
    generate_recursively(Board::empty(), move_count, &mut result);
    result.into_iter().collect()
}

fn encode_key_value(key: u64, value: i32) -> u64 {
    (key << 8) | ((value as u8) as u64)
}

fn decode_key_value(code: u64) -> (u64, i32) {
    (code >> 8, (((code & 0xff) as u8) as i8) as i32)
}

#[derive(Serialize, Deserialize)]
pub struct Book {
    table: Vec<u64>,
}

impl Book {
    pub fn generate(move_count: u32, log_progress: bool) -> Self {
        if log_progress {
            println!("Generating boards...");
        }
        let boards = generate_opening_boards(move_count);
        let count = boards.len();
        if log_progress {
            println!("{} boards generated, now solving.", count);
        }
        let start = Instant::now();
        let mut solver = Solver::new();
        Self {
            table: boards
                .iter()
                .enumerate()
                .flat_map(|(index, board)| {
                    if log_progress && index > 0 && (index % 100 == 0) {
                        let average_duration = (Instant::now() - start).div(index as u32);
                        let remaining = average_duration * (count - index) as u32;
                        println!("Solving position {} out of {}. Average duration {:?}, {:?} remaining.", index, count, average_duration, remaining);
                    }
                    match solver.solve(board.0) {
                        Solution::Solved { score, .. } => {
                            Some(encode_key_value(board.0.key(), score))
                        }
                        _ => None,
                    }
                })
                .collect(),
        }
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;
        let data = bincode::serialize(self).unwrap();
        file.write_all(&data)
    }
}

pub struct LoadedBook {
    table: HashMap<u64, i32>,
}

impl LoadedBook {
    pub fn load_from(book: &Book) -> Self {
        Self {
            table: book
                .table
                .iter()
                .flat_map(|code| {
                    let (key, value) = decode_key_value(*code);
                    [(key, value), (Board::symmetric_key(key), value)]
                })
                .collect(),
        }
    }
}
