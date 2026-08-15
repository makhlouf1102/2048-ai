pub const SIZE: usize = 4;
pub const CELL_COUNT: usize = SIZE * SIZE;

pub type UTile = u32;
pub type Row = [UTile; SIZE];
pub type Matrix = [Row; SIZE];
