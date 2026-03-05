#[derive(Debug, Clone, Copy)]
pub struct Stats {
    /// fixed-point integer (1000 = 1)
    pub attack: i64,
    /// fixed-point integer (1000 = 1)
    pub defense: i64,
    /// fixed-point integer (1000 = 1)
    pub max_hp: i64,
    pub range: u8,
    pub cost: u8,
}
