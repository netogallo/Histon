pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    use super::super::histon::{Relation, relation};

    #[test]
    fn test_select() {

        let columns = vec![String::from("one"), String::from("two")];
        let values = vec![(1,2),(3,4),(5,6)];
        let select_fn: fn(&u32, &u32) -> u32 = |one,two| { one + two };
        let expected = values.iter().map(|(v1, v2)| { select_fn(v1,v2)});
        
        let relation = relation::from_iterable(&columns, values.clone());
        let select = relation.try_select(
            &columns,
            select_fn
        );

        for (actual, expected) in select.unwrap().values.iter().zip(expected) {
            assert_eq!(actual.clone(), expected);
        }
    }
}
