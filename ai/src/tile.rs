use crate::types::*;


trait ITile {
    fn is_empty(&self) -> bool;
}

struct Tile {
    val: UTile,
}


impl ITile for Tile {
    fn is_empty(&self) -> bool {
        self.val == 0
    }
}