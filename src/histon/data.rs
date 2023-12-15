use std::{any::Any, collections::HashMap, ops::{Bound, RangeBounds}};

use super::foundations::*;
use super::foundations::selection::*;

pub struct StaticRelation {
    pub columns : HashMap<String, Box<dyn Any>>
}

impl Relation for StaticRelation {

    type RelationColumn<'t> = RelationColumnLinearRange<'t>;

    fn iter_selection<'t>(
        &'t self,
        column : &String,
        range : Option<RelationColumnSelectionDyn>
    ) -> Self::RelationColumn<'t> {
        
        match self.columns.get_key_value(column) {
            Some ((k,v)) => 
            RelationColumnLinearRange::from_vector_ref(k, v.as_ref(), range),
            _ => panic!("not implemented")
        }
    }
}