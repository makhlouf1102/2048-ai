pub const SIZE: usize = 4;
pub const CELL_COUNT: usize = SIZE * SIZE;

pub type TileRank = u8;
pub type Row = [TileRank; SIZE];
pub type Matrix = [Row; SIZE];

pub fn rank_from_value(value: u32) -> TileRank {
    if value == 0 {
        return 0;
    }

    debug_assert!(value.is_power_of_two());
    value.ilog2() as TileRank
}

pub fn value_from_rank(rank: TileRank) -> u32 {
    1_u32.checked_shl(u32::from(rank)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_round_trip_through_compact_ranks() {
        for value in [0, 2, 4, 8, 128, 2048, 65_536] {
            let rank = rank_from_value(value);
            assert_eq!(if rank == 0 { 0 } else { value_from_rank(rank) }, value);
        }
    }

    #[test]
    fn matrix_uses_one_byte_per_cell() {
        assert_eq!(std::mem::size_of::<Matrix>(), CELL_COUNT);
    }
}
