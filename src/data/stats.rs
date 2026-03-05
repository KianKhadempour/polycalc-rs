#[derive(Debug, Clone, Copy)]
pub struct Stats {
    /// fixed-point integer (10 = 1)
    pub attack: u16,
    /// fixed-point integer (10 = 1)
    pub defense: u16,
    /// fixed-point integer (10 = 1)
    pub max_hp: u16,
    pub range: u8,
    pub cost: u8,
}
