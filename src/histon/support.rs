use core::any::Any;
use std::collections::HashMap;

type DynamicColumnMap = HashMap<String, Box<dyn Any>>

pub trait ToColumn {

    fn init_columns(
        columns: &Vec<String>
    ) -> DynamicColumnMap;

    fn add_column(
        value : Self,
        columns : &mut DynamicColumnMap
    );
}

impl<A1,A2> ToColumn for (A1,A2) {

    fn init_columns(
        columns: &Vec<String>
    ) -> DynamicColumnMap {

        let c1: Vec<A1> = Vec::new();
        let c2: Vec<A2> = Vec::new();
        let mut result: DynamicColumnMap = HashMap::new();

        result.insert(columns[0], Box::new(c1));

        return result
    }

    fn add_column(
        (v1, v2) : Self,
        columns : &mut DynamicColumnMap
    ) {


    }
}