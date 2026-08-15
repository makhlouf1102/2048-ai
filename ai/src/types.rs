pub const SIZE: usize = 4;

pub type UTile = u32;
pub type Row = [UTile; SIZE];
pub type Matrix = [Row; SIZE];

pub type Position = (u8, u8);

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}