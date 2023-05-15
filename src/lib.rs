use crate::board::Board;
use crate::move_sorter::MoveSorter;
use crate::transposition_table::{TranspositionTable, TranspositionTableValue};

pub mod board;
mod move_sorter;
mod transposition_table;
mod book;

const COLUMN_ORDER: [u32; Board::WIDTH as usize] = [3, 2, 4, 1, 5, 0, 6];

fn negamax(
    board: Board,
    mut alpha: i32,
    mut beta: i32,
    transposition_table: &mut TranspositionTable,
    node_counter: &mut u64,
) -> i32 {
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
    
    let max = (Board::SQUARES as i32 - 1 - board.filled_squares() as i32) / 2; // Upper bound of our score as we cannot win immediately
    if beta > max {
        beta = max; //There is no need to keep beta above our max possible score.
        if alpha >= beta {
            return beta; //Prune the exploration
        }
    }
    
    //Look in the transposition table
    if let Some(value) = transposition_table.get(board) {
        match value {
            TranspositionTableValue::UpperBound(max) => {
                if beta > max {
                    beta = max;
                    if alpha >= beta {
                        return beta;
                    }
                }
            }
            TranspositionTableValue::LowerBound(min) => {
                if alpha < min {
                    alpha = min;
                    if alpha >= beta {
                        return alpha;
                    }
                }
            }
        }
    }

    let mut move_sorter = MoveSorter::new();
    for &column in COLUMN_ORDER.iter().rev() {
        let move_mask = Board::column_mask(column) & possible;
        if move_mask != 0 {
            let new_board = unsafe { board.make_move_unchecked(move_mask) };
            unsafe { move_sorter.add(move_mask, new_board.opponent_heuristic_score()) };
        }
    }

    while let Some(move_mask) = move_sorter.get_next() {
        let new_board = unsafe { board.make_move_unchecked(move_mask) };
        let score = -negamax(new_board, -beta, -alpha, transposition_table, node_counter);
        if score >= beta {
            transposition_table.set(board, TranspositionTableValue::LowerBound(score));
            return score;
        }
        if score > alpha {
            alpha = score;
        }
    }

    transposition_table.set(board, TranspositionTableValue::UpperBound(alpha));
    alpha
}

fn search(
    board: Board,
    transposition_table: &mut TranspositionTable,
    node_counter: &mut u64,
) -> i32 {
    let mut min = -(Board::SQUARES as i32 - board.filled_squares() as i32) / 2;
    let mut max = (Board::SQUARES as i32 + 1 - board.filled_squares() as i32) / 2;

    while min < max {
        let mut med = min + (max - min) / 2;
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

pub enum Solution {
    Draw,
    Victory,
    Solved { score: i32, nodes_explored: u64 },
}

pub struct Solver {
    transposition_table: TranspositionTable,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            transposition_table: TranspositionTable::new(),
        }
    }

    pub fn solve(&mut self, board: Board) -> Solution {
        if board.is_full() {
            Solution::Draw
        } else if board.is_victory() {
            Solution::Victory
        } else {
            let mut node_counter = 0;
            let score = if board.has_winning_move() {
                (Board::SQUARES as i32 + 1 - board.filled_squares() as i32) / 2
            } else {
                search(board, &mut self.transposition_table, &mut node_counter)
            };
            Solution::Solved {
                score,
                nodes_explored: node_counter,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use crate::book::Book;

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
            let computed_score = match solver.solve(board) {
                Solution::Solved {
                    score,
                    nodes_explored,
                } => {
                    node_counter += nodes_explored;
                    score
                }
                _ => panic!(),
            };
            total_duration += Instant::now() - start;
            counter += 1;
            assert_eq!(computed_score, score);
        }
        let average_duration = total_duration.div_f32(counter as f32);
        println!(
            "Average duration: {:?}, nodes: {}",
            average_duration,
            node_counter as f64 / counter as f64
        );
    }
    
    #[test]
    fn generate_book() {
        Book::generate(10, true).save_to_file("./opening_book_10_moves").unwrap();
    }
    
    /*#[test]
    fn test_initial_position() {
        let mut solver = Solver::new();
        let start = Instant::now();
        match solver.solve(Board::empty()) {
            Solution::Solved { score, .. } => {
                assert_eq!(score, 1);
                println!("Completed in {:?}", Instant::now() - start);
            }
            _ => panic!()
        }
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
    }*/
}
