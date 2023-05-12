use crate::Board;

#[derive(Copy, Clone)]
struct Entry {
    move_mask: u64,
    heuristic_score: u32
}

pub struct MoveSorter {
    size: usize,
    entries: [Entry; Board::WIDTH as usize],
}

impl MoveSorter {
    pub fn new() -> Self {
        Self {
            size: 0,
            entries: [Entry {
                move_mask: 0,
                heuristic_score: 0
            }; Board::WIDTH as usize]
        }
    }

    pub fn add(&mut self, move_mask: u64, heuristic_score: u32) {
        let mut pos = self.size;
        while pos > 0 && self.entries[pos - 1].heuristic_score > heuristic_score {
            self.entries[pos] = self.entries[pos - 1];
            pos -= 1;
        }
        self.entries[pos].move_mask = move_mask;
        self.entries[pos].heuristic_score = heuristic_score;
        self.size += 1;
    }
    
    pub fn get_next(&mut self) -> Option<u64> {
        if self.size > 0 {
            self.size -= 1;
            Some(self.entries[self.size].move_mask)
        } else {
            None
        }
    }
}
