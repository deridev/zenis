use crate::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArenaPaymentMethod {
    User(Id<UserMarker>),
    EveryoneHalf,
}

pub const PRICE_PER_ARENA: i64 = 25;
pub const PRICE_PER_ACTION: i64 = 10;
