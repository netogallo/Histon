use std::{any::Any, collections::HashMap};
use std::rc::Rc;

use super::foundations::*;


struct StaticRelation {
    columns : HashMap<String, Rc<dyn Any>>
}

impl Relation for StaticRelation {

    fn try_select<FOut, Args, TResult>(
        &self,
        columns : &Vec<String>,
        select : FOut
    ) -> RelationResult<SelectIterator<TResult>>
    where
        FOut : SelectDispatchFn<Args, TResult> {

    
        let args =
            columns.iter()
            .map(|col| { self.columns[col].clone() })
            .collect();
    
        return select.dispatch(columns, &args)
    }
}