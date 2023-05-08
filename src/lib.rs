use crate::board::Board;

pub mod board;

const COLUMN_ORDER: [usize; 7] = [3, 2, 4, 1, 5, 0, 6];

fn negamax(board: Board, mut alpha: i32, mut beta: i32) -> i32 {
    if board.is_full() {
        return 0;
    }
    
    if board.has_winning_move() {
        return (Board::SQUARES as i32 + 1 - board.filled_squares() as i32) / 2;
    }

    let max = (Board::SQUARES as i32 - 1 - board.filled_squares() as i32) / 2; // Upper bound of our score as we cannot win immediately
    if beta > max {
        beta = max;
        if alpha >= beta {
            return beta;
        }
    }

    for column in COLUMN_ORDER {
        if let Ok(new_board) = board.make_move(column) {
            let score = -negamax(new_board, -beta, -alpha);
            if score >= beta {
                return score;
            }
            if score > alpha {
                alpha = score;
            }
        }
    }
    alpha
}

fn solve(board: Board) -> i32 {
    negamax(board, -(Board::SQUARES as i32), Board::SQUARES as i32)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use super::*;
    
    fn test(file: &str) {
        let mut total_duration = Duration::new(0, 0);
        let mut counter = 0;
        for line in file.lines() {
            let mut split = line.split_whitespace();
            let (position, score) = (split.next().unwrap(), split.next().unwrap());
            let board = Board::from_string(position).unwrap();
            let score: i32 = score.parse().unwrap();
            let start = Instant::now();
            let computed_score = solve(board);
            total_duration += Instant::now() - start;
            counter += 1;
            assert_eq!(computed_score, score);
        }
        let average_duration = total_duration.div_f32(counter as f32);
        println!("Average duration: {:?}", average_duration);
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
