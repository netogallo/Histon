mod data;
mod foundations;
mod extend;
mod support;

pub use foundations::Relation;

pub mod relation {

    use core::any::Any;
    use std::collections::HashMap;

    use super::data::StaticRelation;
    use super::support::ToColumn;

    pub fn from_iterable<Item>(
        columns : &Vec<String>,
        values : Vec<Item>
    ) -> StaticRelation
    where Item : ToColumn {

        let mut data: HashMap<String, Box<dyn Any>> = HashMap::new();

        for column in columns {

            // let value : Box<dyn Any> = Box::new(vec());
            // data.insert(column.clone(), vec())
        }

        panic!("")
    }
}