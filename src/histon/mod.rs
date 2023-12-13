mod data;
mod extend;
mod foundations;
mod join;
mod support;

pub use foundations::Relation;

pub mod relation {

    use super::data::StaticRelation;
    use super::support::ToColumn;

    pub fn from_iterable<Item>(
        columns : &Vec<String>,
        values : Vec<Item>
    ) -> StaticRelation
    where Item : ToColumn {

        let mut data = <Item as ToColumn>::init_columns(columns);

        for value in values {
            <Item as ToColumn>::add_column(value, &mut data);
        }

        return StaticRelation{ 
            columns : data.to_hash_map()
        };
    }
}