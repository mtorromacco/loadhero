use std::collections::HashMap;

pub struct StatsInfo {
    pub total_time: u128,
    pub min_time: u128,
    pub max_time: u128,
    pub status_codes: HashMap<u16, u32>,
    pub points: Vec<(f32, f32)>
}