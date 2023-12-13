use std::{any::Any, collections::HashMap, ops::{Bound, RangeBounds}};

use super::foundations::*;

pub struct StaticRelation {
    pub columns : HashMap<String, Box<dyn Any>>
}

impl Relation for StaticRelation {

    fn iter_range<TRange>(&self, column : &String, range : TRange) -> RelationColumnRange<'_>
            where TRange : RangeBounds<dyn Any> {
        
        match (self.columns.get_key_value(column), range.start_bound(), range.end_bound()) {
            (Some ((k,v)), Bound::Unbounded, Bound::Unbounded) => 
                RelationColumnRange::from_vector_ref(k, v.as_ref()),
            _ => panic!("not implemented")
        }
    }
}