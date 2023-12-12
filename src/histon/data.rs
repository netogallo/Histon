use std::{any::Any, collections::HashMap};

use super::foundations::*;

pub struct StaticRelation {
    columns : HashMap<String, Box<dyn Any>>
}

impl Relation for StaticRelation {

    fn try_select<F, TResult>(
        &self,
        columns : &Vec<String>,
        select : F
    ) -> RelationResult<SelectResult<TResult>>
    where
        F : SelectDispatchFn<TResult> {


        let args =
            columns.iter()
            .map(|col| { self.columns[col].as_ref() })
            .collect();

    
        return select.dispatch(columns, &args)
    }
}