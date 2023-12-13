use core::any::Any;
use std::{collections::HashMap, ops::RangeFull};

pub struct RelationBuilder {
    column_names : Vec<String>,
    column_values : Vec<Box<dyn Any>>
}

impl RelationBuilder {

    pub fn to_hash_map(
        &mut self
    ) -> HashMap<String, Box<dyn Any>> {

        let mut result = HashMap::new();
        let keys = self.column_names.drain(RangeFull);
        let values = self.column_values.drain(RangeFull);

        for (key, value) in keys.zip(values) {
            result.insert(key, value);
        }

        return result;
    }
}

pub trait ToColumn {

    fn init_columns(
        columns: &Vec<String>
    ) -> RelationBuilder;

    fn add_column(
        value : Self,
        columns : &mut RelationBuilder
    );
}

impl<A1,A2> ToColumn for (A1,A2)
    where
        A1 : Any,
        A2 : Any {

    fn init_columns(
        columns: &Vec<String>
    ) -> RelationBuilder {

        let c1: Vec<A1> = Vec::new();
        let c2: Vec<A2> = Vec::new();

        return RelationBuilder {
            column_names: columns.clone(),
            column_values: vec![Box::new(c1), Box::new(c2)]
        };
    }

    fn add_column(
        (v1, v2) : Self,
        columns : &mut RelationBuilder
    ) {
        if let [a1_box, a2_box] = &mut columns.column_values[0..2] {
            let a1_values = a1_box.downcast_mut::<Vec<A1>>();
            let a2_values = a2_box.downcast_mut::<Vec<A2>>();
            match (a1_values, a2_values) {
                (Some(a1_vec), Some(a2_vec)) => {
                    a1_vec.push(v1);
                    a2_vec.push(v2);
                }
                _ => panic!("The relation builder is used on an invalid column type!")
            }
        }
        else {
            panic!("The relation builder has the wrong number of columns!")
        }

    }
}