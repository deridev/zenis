pub mod brain;
pub mod claude_brain;
pub mod cohere_brain;
pub mod common;
pub mod util;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BrainType {
    Claude,
    Cohere,
}

use brain::Brain;
impl BrainType {
    pub fn get(&self) -> Box<dyn Brain + Send + Sync + 'static> {
        match self {
            Self::Claude => Box::new(claude_brain::ClaudeBrain),
            Self::Cohere => Box::new(cohere_brain::CohereBrain),
        }
    }
}
