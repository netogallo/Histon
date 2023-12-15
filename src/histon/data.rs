use std::{any::Any, collections::HashMap, ops::{Bound, RangeBounds}};

use super::foundations::*;
use super::foundations::selection::*;

pub struct StaticRelation {
    pub columns : HashMap<String, Box<dyn Any>>
}

impl Relation for StaticRelation {

    fn iter_selection(
        &self,
        column : &String,
        range : Option<RelationColumnSelectionDyn>
    ) -> RelationColumnRange<'_> {
        
        match self.columns.get_key_value(column) {
            Some ((k,v)) => 
                RelationColumnRange::from_vector_ref(k, v.as_ref(), range),
            _ => panic!("not implemented")
        }
    }
}