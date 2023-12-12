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
        let relation = relation::from_iterable(&columns, values);
        let select = relation.try_select::<_,(&u32, &u32),_>(
            &columns,
            |(one,two)| { one + two }
        );
    }
}
