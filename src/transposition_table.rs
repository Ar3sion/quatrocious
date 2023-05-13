use std::collections::hash_map::DefaultHasher;
use crate::Board;

#[derive(Copy, Clone)]
struct Entry(u64);

impl Entry {
    pub fn new(key: u64, value: i32) -> Self {
        let signed = value as i8;
        let unsigned = signed as u8;
        Self((key << 8) | (unsigned as u64))
    }
    
    pub fn get_key(self) -> u64 {
        self.0 >> 8
    }
    
    pub fn get_value(self) -> i32 {
        let unsigned = (self.0 & 0xff) as u8;
        let signed = unsigned as i8;
        signed as i32
    }
}

pub struct TranspositionTable {
    table: Vec<Entry>
}

impl TranspositionTable {
    const SIZE: usize = 8388593; //(64MB) This is a prime number to reduce collisions
    
    pub fn new() -> Self {
        Self {
            table: vec![Entry::new(0xffffffffffffff, 0); Self::SIZE] //0xffffffffffffff does not represent any board
        }
    }
    
    fn index(key: u64) -> usize {
        (key % (Self::SIZE as u64)) as usize
    }
    
    pub fn set(&mut self, board: Board, score: i32) {
        let key = board.key();
        self.table[Self::index(key)] = Entry::new(key, score);
    }
    
    pub fn get(&mut self, board: Board) -> Option<i32> {
        let key = board.key();
        let entry = self.table[Self::index(key)];
        if entry.get_key() == key {
            Some(entry.get_value())
        } else {
            None
        }
    }
}