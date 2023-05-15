use crate::Board;

pub enum TranspositionTableValue {
    UpperBound(i32),
    LowerBound(i32),
}

#[derive(Copy, Clone)]
struct Entry(u64);

impl Entry {
    pub fn new(key: u64, value: TranspositionTableValue) -> Self {
        let value = match value {
            TranspositionTableValue::UpperBound(score) => ((score as u8) as u64) << 1,
            TranspositionTableValue::LowerBound(score) => (((score as u8) as u64) << 1) | 1,
        };
        Self((key << 9) | value)
    }

    pub fn get_key(self) -> u64 {
        self.0 >> 9
    }

    fn extract_score(&self) -> i32 {
        let byte = ((self.0 >> 1) & 0xff) as u8;
        (byte as i8) as i32
    }

    pub fn get_value(self) -> TranspositionTableValue {
        if self.0 & 1 == 0 {
            TranspositionTableValue::UpperBound(self.extract_score())
        } else {
            TranspositionTableValue::LowerBound(self.extract_score())
        }
    }
}

pub struct TranspositionTable {
    table: Vec<Entry>,
}

impl TranspositionTable {
    const SIZE: usize = 16777259; //(64MB) This is a prime number to reduce collisions

    pub fn new() -> Self {
        Self {
            table: vec![
                Entry::new(0xffffffffffffff, TranspositionTableValue::UpperBound(0));
                Self::SIZE
            ], //0xffffffffffffff does not represent any board
        }
    }

    fn index(key: u64) -> usize {
        (key % (Self::SIZE as u64)) as usize
    }

    pub fn set(&mut self, board: Board, value: TranspositionTableValue) {
        let key = board.key();
        self.table[Self::index(key)] = Entry::new(key, value);
    }

    pub fn get(&self, board: Board) -> Option<TranspositionTableValue> {
        let key = board.key();
        let entry = self.table[Self::index(key)];
        if entry.get_key() == key {
            Some(entry.get_value())
        } else {
            None
        }
    }
}
