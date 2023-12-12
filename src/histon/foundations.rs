use std::any::{Any, TypeId};
use std::slice::Iter;
use std::iter::{Map, Zip};

pub enum RelationError {
    IncorrectColumnType { column : String, type_id : TypeId },
    IncorrectColumnCount { expected : usize, actual : usize }
}

pub type RelationResult<TResult> = Result<TResult, RelationError>;

impl RelationError {

    pub fn raise_incorrect_column_count<T>(
        expected : usize,
        actual : usize
    ) -> RelationResult<T> {

        Result::Err(Self::IncorrectColumnCount{ expected, actual })
    }

    pub fn raise_incorrect_column_type<T>(
        column : &String,
    ) -> RelationResult<T>
    where T : Any {
        return Result::Err(RelationError::IncorrectColumnType { column : column.clone(), type_id: TypeId::of::<T>() })
    }

    pub fn incorrect_column_type<T>(
        column : String,
    ) -> RelationError
    where T : Any {
        return RelationError::IncorrectColumnType { column, type_id: TypeId::of::<T>() }
    }
}

pub trait ToArgs : Sized {

    type Item<'a>;
    type Iter<'a> : Iterator<Item = Self::Item<'a>>;
    
    fn to_args<'a>(
        columns : &Vec<String>,
        args : &'a Vec<&'a dyn Any>,
    ) -> RelationResult<Self::Iter<'a>>;
}

pub struct SelectResult<Values> {
    pub values : Vec<Values>
}

impl<TValue> FromIterator<TValue> for SelectResult<TValue> {

    fn from_iter<T: IntoIterator<Item = TValue>>(iter: T) -> Self {

        return SelectResult{ values : iter.into_iter().collect() };
    }
}

pub trait SelectDispatchFn<Args, TResult> {

    fn dispatch<'a>(
        &self,
        columns : &Vec<String>,
        args: &'a Vec<&'a dyn Any>
    ) -> RelationResult<SelectResult<TResult>>
    where
        Args : ToArgs,
        Self : Fn(<Args as ToArgs>::Item<'a>) -> TResult;
}

fn iter_from_known_collection<'a, TValue>(
    column : &String,
    collection : &'a dyn Any
) -> RelationResult<Iter<'a, TValue>>
    where TValue : Any {

    let try_vec = collection.downcast_ref::<Vec<TValue>>().map(|v| { v.iter() });

    return try_vec
        .ok_or_else(|| { RelationError::incorrect_column_type::<Iter<TValue>>(column.clone()) })
}

impl<A1> ToArgs for (&'_ A1,) where A1 : Any {

    type Item<'a> = (&'a A1,);
    type Iter<'a> = Map<Iter<'a, A1>, fn(&A1) -> (&A1,)>;

    fn to_args<'a>(
        columns : &Vec<String>,
        args : &'a Vec<&'a dyn Any>
    ) -> RelationResult<Self::Iter<'a>> {

        if args.len() != 1 {
            return RelationError::raise_incorrect_column_count(1,columns.len())
        }

        return iter_from_known_collection(&columns[0], args[0])
            .map(|v| {
                let mapper : fn(&A1) -> (&A1,) = |v| { (v,)};
                return v.map(mapper);
            })
            .or_else(|_e| RelationError::raise_incorrect_column_type(&columns[0]));
    }
}

impl<A1,A2> ToArgs for (&'_ A1,&'_ A2)
    where
        A1 : Any,
        A2 : Any
    {

    type Item<'a> = (&'a A1, &'a A2);
    type Iter<'a> = Zip<Iter<'a, A1>, Iter<'a, A2>>;

    fn to_args<'a>(
        columns : &Vec<String>,
        args : &'a Vec<&'a dyn Any>,
    ) -> RelationResult<Self::Iter<'a>> {

        if args.len() != 2 {
            return RelationError::raise_incorrect_column_count(2, columns.len())
        }

        let arg1 = iter_from_known_collection::<A1>(&columns[0], args[0]);
        
        let arg2 = iter_from_known_collection::<A2>(&columns[1], args[1]);

        return arg1.and_then(|a1| { 
            arg2.map(|a2|{
                a1.zip(a2)
            })
        });
    }
}

impl<F, Args, TResult> SelectDispatchFn<Args, TResult> for F {
    
    fn dispatch<'a>(
        &self,
        columns: &Vec<String>,
        args: &'a Vec<&'a dyn Any>
    ) -> RelationResult<SelectResult<TResult>>
    where
        Args : ToArgs,
        Self : Fn(<Args as ToArgs>::Item<'a>) -> TResult {
        
        let to_args_result = <Args as ToArgs>::to_args(columns, args);
        return to_args_result
            .map(|args| {
                args.map(|v| { self(v) }).collect()
            }
        )
    }
}

pub trait Relation {
    fn try_select<F, Args, TResult>(
        &self,
        columns : &Vec<String>,
        select : F
    ) -> RelationResult<SelectResult<TResult>>
    where
        Args : ToArgs,
        F : Fn(<Args as ToArgs>::Item<'_>) -> TResult,
        F : SelectDispatchFn<Args, TResult>;
}