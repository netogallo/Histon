use std::{any::Any, collections::HashMap};

use super::foundations::*;

struct StaticRelation {
    columns : HashMap<String, Box<dyn Any>>
}

impl Relation for StaticRelation {

    fn try_select<FOut, Args, TResult>(
        &self,
        columns : &Vec<String>,
        select : FOut
    ) -> RelationResult<SelectResult<TResult>>
    where
        Args : ToArgs,
        FOut : Fn(<Args as ToArgs>::Item<'_>) -> TResult,
        FOut : SelectDispatchFn<Args, TResult> {


        let args =
            columns.iter()
            .map(|col| { self.columns[col].as_ref() })
            .collect();

    
        return select.dispatch(columns, &args)
    }
}