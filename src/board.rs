/*
 connect4.c, ia_main.c
*/

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Player {
    White, //Yellow in the GUI
    Black //Red in the GUI
}

impl Player {
    pub fn opponent(self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Square {
    Empty,
    Taken(Player)
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Board {
    current_player: u64,
    mask: u64,
    filled: usize
}

impl Board {
    pub const WIDTH: usize = 7;
    pub const HEIGHT: usize = 6;
    pub const SQUARES: usize = Self::WIDTH * Self::HEIGHT;

    const fn bottom(width: usize) -> u64 {
        if width == 0 {
            0
        } else {
            Self::bottom(width - 1) | (1 << ((width - 1) * (Self::HEIGHT + 1)))
        }
    }

    const BOTTOM: u64 = Self::bottom(Self::WIDTH);
    const BOARD_MASK: u64 = Self::BOTTOM * ((1 << Self::HEIGHT) - 1);

    fn column_mask(column: usize) -> u64 {
        ((1 << Self::HEIGHT) - 1) << (column * (Self::HEIGHT + 1))
    }
    
    fn row_mask(row: usize) -> u64 {
        Self::BOTTOM << row
    }
    
    fn square_mask(column: usize, row: usize) -> u64 {
        Self::column_mask(column) & Self::row_mask(row)
    }
    
    pub fn new() -> Self {
        Self {
            current_player: 0,
            mask: 0,
            filled: 0
        }
    }
    
    pub fn from_string(string: &str) -> Result<Self, String> {
        let mut board = Self::new();
        for character in string.chars() {
            if board.is_endgame() {
                return Err(String::from("Too many moves!"))
            }
            let digit = character.to_digit(10).ok_or(String::from("The character is not a digit!"))?;
            if !(1..=7).contains(&digit) {
                return Err(String::from("The digit is not in the range 1..=7"))
            }
            let column = (digit - 1) as usize;
            board = board.make_move(column).map_err(|_| String::from("Invalid move!"))?;
        }
        Ok(board)
    }
    
    pub fn player_to_play(self) -> Player {
        if self.filled & 1 == 0 {
            Player::White //Move count is even: white to play
        } else {
            Player::Black //Move count is odd: black to play
        }
    }
    
    pub fn is_winning(self) -> bool {
        let last_player = self.mask ^ self.current_player;

        let horizontal_overlap_1 = last_player & (last_player >> (Self::HEIGHT + 1));
        let horizontal_overlap_2 = horizontal_overlap_1 & (horizontal_overlap_1 >> (2 * (Self::HEIGHT + 1)));
        if horizontal_overlap_2 != 0 {
            return true;
        }

        let diagonal1_overlap_1 = last_player & (last_player >> Self::HEIGHT);
        let diagonal1_overlap_2 = diagonal1_overlap_1 & (diagonal1_overlap_1 >> (2 * Self::HEIGHT));
        if diagonal1_overlap_2 != 0 {
            return true;
        }

        let diagonal2_overlap_1 = last_player & (last_player >> (Self::HEIGHT + 2));
        let diagonal2_overlap_2 = diagonal2_overlap_1 & (diagonal2_overlap_1 >> (2 * (Self::HEIGHT + 2)));
        if diagonal2_overlap_2 != 0 {
            return true;
        }

        let vertical_overlap_1 = last_player & (last_player >> 1);
        let vertical_overlap_2 = vertical_overlap_1 & (vertical_overlap_1 >> 2);
        if vertical_overlap_2 != 0 {
            return true;
        }

        false
    }
    
    fn winning_positions(self, player: Player) -> u64 {
        let player_positions = if self.player_to_play() == player {
            self.current_player
        } else {
            self.current_player ^ self.mask
        };

        //Vertical
        let mut result = (player_positions << 1) & (player_positions << 2) & (player_positions << 3);

        //Horizontal
        let one_left = player_positions >> (Self::HEIGHT + 1);
        let one_right = player_positions << (Self::HEIGHT + 1);
        let two_left = one_left & (player_positions >> (2 * (Self::HEIGHT + 1)));
        let two_right = one_right & (player_positions << (2 * (Self::HEIGHT + 1)));
        result |= two_right & (player_positions << (3 * (Self::HEIGHT + 1)));
        result |= two_right & one_left;
        result |= two_left & (player_positions >> (3 * (Self::HEIGHT + 1)));
        result |= two_left & one_right;

        //Diagonal 1
        let one_top_left = player_positions >> Self::HEIGHT;
        let one_bottom_right = player_positions << Self::HEIGHT;
        let two_top_left = one_top_left & (player_positions >> (2 * Self::HEIGHT));
        let two_bottom_right = one_bottom_right & (player_positions << (2 * Self::HEIGHT));
        result |= two_bottom_right & (player_positions << (3 * Self::HEIGHT));
        result |= two_bottom_right & one_top_left;
        result |= two_top_left & (player_positions >> (3 * Self::HEIGHT));
        result |= two_top_left & one_bottom_right;

        //Diagonal 2
        let one_bottom_left = player_positions >> (Self::HEIGHT + 2);
        let one_top_right = player_positions << (Self::HEIGHT + 2);
        let two_bottom_left = one_bottom_left & (player_positions >> (2 * (Self::HEIGHT + 2)));
        let two_top_right = one_top_right & (player_positions << (2 * (Self::HEIGHT + 2)));
        result |= two_top_right & (player_positions << (3 * (Self::HEIGHT + 2)));
        result |= two_top_right & one_bottom_left;
        result |= two_bottom_left & (player_positions >> (3 * (Self::HEIGHT + 2)));
        result |= two_bottom_left & one_top_right;

        result &= Self::BOARD_MASK; //Only valid cells
        result &= Self::BOTTOM + self.mask; //Only top cells
        
        result
    }
    
    pub fn has_winning_move(self) -> bool {
        self.winning_positions(self.player_to_play()) != 0
    }
    
    pub fn is_full(self) -> bool {
        self.filled == Self::SQUARES
    }
    
    pub fn is_endgame(self) -> bool {
        self.is_winning() || self.is_full()
    }
    
    pub fn filled_squares(self) -> usize {
        self.filled
    }
    
    pub fn can_play(self, column: usize) -> bool {
        if column >= Self::WIDTH {
            return false;
        }
        let top_mask = (1 << Self::HEIGHT - 1) << (column * (Self::HEIGHT + 1));
        self.mask & top_mask == 0
    }
    
    pub fn make_move(self, column: usize) -> Result<Board, ()> {
        if !self.can_play(column) {
            return Err(())
        }
        Ok(Self {
            current_player: self.current_player ^ self.mask, //flip all values
            mask: self.mask | (self.mask + (1 << (column * (Self::HEIGHT + 1)))), //extend mask
            filled: self.filled + 1
        })
    }
    
    pub fn get_square(&self, column: usize, row: usize) -> Option<Square> {
        if column >= Self::WIDTH || row >= Self::HEIGHT {
            None
        } else {
            let square_mask = Self::square_mask(column, row);
            Some(if self.mask & square_mask == 0 {
                Square::Empty
            } else {
                Square::Taken(if self.current_player & square_mask != 0 {
                    self.player_to_play()
                } else {
                    self.player_to_play().opponent()
                })
            })
        }
    }
    
    fn symmetric(values: u64) -> u64 {
        let mut result = 0;
        for column in 0..Self::WIDTH {
            let target_column = Self::WIDTH - 1 - column;
            let selected = values & Self::column_mask(column);
            if target_column < column { //We shift left on the board: that's a right shift
                let offset = column - target_column;
                result |= selected >> (offset * (Self::HEIGHT + 1));
            } else {
                let offset = target_column - column;
                result |= selected << (offset * (Self::HEIGHT + 1));
            }
        }
        result
    }
    
    pub fn symmetric_board(self) -> Self {
        Self {
            current_player: Self::symmetric(self.current_player),
            mask: Self::symmetric(self.mask),
            filled: self.filled
        }
    }
    
    pub fn key(self) -> u64 {
        self.current_player + self.mask
    }
}