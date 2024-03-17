use std::ops::RangeInclusive;

pub const BOT_IDS: &[u64] = &[1215409249262379018];
pub const DEBUG_GUILD_ID: u64 = 562364424002994189;

pub const DEBUG: bool = false;

pub const ARENA_NAME_SIZE: RangeInclusive<usize> = 1..=32;
pub const ARENA_DESCRIPTION_SIZE: RangeInclusive<usize> = 1..=300;

pub const NAME_SIZE: RangeInclusive<usize> = 1..=16;
pub const DESCRIPTION_SIZE: RangeInclusive<usize> = 1..=800;
pub const INTRODUCTION_MESSAGE_SIZE: RangeInclusive<usize> = 1..=256;
