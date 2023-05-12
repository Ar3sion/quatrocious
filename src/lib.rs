use crate::board::Board;
use crate::move_sorter::MoveSorter;
use crate::transposition_table::TranspositionTable;

pub mod board;
mod transposition_table;
mod move_sorter;

const COLUMN_ORDER: [u32; Board::WIDTH as usize] = [3, 2, 4, 1, 5, 0, 6];

fn negamax(board: Board, mut alpha: i32, mut beta: i32, transposition_table: &mut TranspositionTable, node_counter: &mut u64) -> i32 {
    *node_counter += 1;
    
    let possible = board.non_losing_moves();
    if possible == 0 { 
        return -(Board::SQUARES as i32 - board.filled_squares() as i32) / 2; //The opponent wins next move
    }
    
    if board.filled_squares() >= Board::SQUARES - 2 {
        return 0; //Draw (we can't win immediately, and we don't lose after playing either)
    }

    let min = -(Board::SQUARES as i32 - 2 - board.filled_squares() as i32) / 2; //Lower bound of score as opponent cannot win next move
    if alpha < min {
        alpha = min; //There is no need to keep alpha under our minimum possible score.
        if alpha >= beta {
            return alpha; //Prune the exploration
        }
    }

    let max = if let Some(score) = transposition_table.get(board) {
        score //The upper bound is in the transposition table
    } else {
        (Board::SQUARES as i32 - 1 - board.filled_squares() as i32) / 2 // Upper bound of our score as we cannot win immediately
    };
    if beta > max {
        beta = max; //There is no need to keep beta above our max possible score.
        if alpha >= beta {
            return beta; //Prune the exploration
        }
    }
    
    let mut move_sorter = MoveSorter::new();
    for column in COLUMN_ORDER {
        let move_mask = Board::column_mask(column) & possible;
        if move_mask != 0 {
            let new_board = unsafe { board.make_move_unchecked(move_mask) };
            move_sorter.add(move_mask, new_board.heuristic_score(board.player_to_play()))
        }
    }
    
    while let Some(move_mask) = move_sorter.get_next() {
        let new_board = unsafe { board.make_move_unchecked(move_mask) };
        let score = -negamax(new_board, -beta, -alpha, transposition_table, node_counter);
        if score >= beta {
            return score;
        }
        if score > alpha {
            alpha = score;
        }
    }
    
    transposition_table.set(board, alpha);
    alpha
}

fn search(board: Board, transposition_table: &mut TranspositionTable, node_counter: &mut u64) -> i32 {
    let mut min = -(Board::SQUARES as i32 - board.filled_squares() as i32) / 2;
    let mut max = (Board::SQUARES as i32 + 1 - board.filled_squares() as i32) / 2;
    
    while min < max {
        let mut med = min + (max - min)/2;
        if med <= 0 && min / 2 < med {
            med = min / 2;
        } else if med >= 0 && max / 2 > med {
            med = max / 2;
        }
        let r = negamax(board, med, med + 1, transposition_table, node_counter); 
        if r <= med {
            max = r;
        } else {
            min = r;
        } 
    }
    
    return min;
}


pub struct Solver {
    transposition_table: TranspositionTable
}

impl Solver {
    pub fn new() -> Self {
        Self {
            transposition_table: TranspositionTable::new()
        }
    }

    pub fn solve(&mut self, board: Board, node_counter: &mut u64) -> i32 {
        if board.has_winning_move() {
            (Board::SQUARES as i32 + 1 - board.filled_squares() as i32) / 2
        } else {
            search(board, &mut self.transposition_table, node_counter)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use super::*;
    
    fn test(file: &str) {
        let mut solver = Solver::new();
        let mut total_duration = Duration::new(0, 0);
        let mut counter = 0;
        let mut node_counter = 0;
        for line in file.lines() {
            let mut split = line.split_whitespace();
            let (position, score) = (split.next().unwrap(), split.next().unwrap());
            let board = Board::from_string(position).unwrap();
            let score: i32 = score.parse().unwrap();
            let start = Instant::now();
            let computed_score = solver.solve(board, &mut node_counter);
            total_duration += Instant::now() - start;
            counter += 1;
            assert_eq!(computed_score, score);
        }
        let average_duration = total_duration.div_f32(counter as f32);
        println!("Average duration: {:?}, nodes: {}", average_duration, node_counter as f64 / counter as f64);
    }

    #[test]
    fn test_end_easy() {
        test(include_str!("./test_sets/Test_L3_R1"))
    }

    #[test]
    fn test_middle_easy() {
        test(include_str!("./test_sets/Test_L2_R1"))
    }

    #[test]
    fn test_middle_medium() {
        test(include_str!("./test_sets/Test_L2_R2"))
    }
    
    #[test]
    fn test_begin_easy() {
        test(include_str!("./test_sets/Test_L1_R1"))
    }

    #[test]
    fn test_begin_medium() {
        test(include_str!("./test_sets/Test_L1_R2"))
    }

    #[test]
    fn test_begin_hard() {
        test(include_str!("./test_sets/Test_L1_R3"))
    }
}
