use std::any::Any;
use std::ops::{Bound, RangeBounds};

#[derive(Clone)]
pub enum RelationColumnSelection<TItem> {
    BoundsSelection { bounds : (Bound<TItem>, Bound<TItem>) },
    AllItems
}

pub type RelationColumnSelectionDyn = RelationColumnSelection<Box<dyn Any>>;

impl<TItem> RelationColumnSelection<TItem> {
    
    pub fn contains(&self, other: &TItem) -> bool
        where TItem : Ord {

        match self {
            Self::BoundsSelection { bounds } =>
                bounds.contains(other),
            Self::AllItems => true
        }
    }
}