use std::any::{Any, TypeId};
use std::ops::{RangeBounds, RangeFull};
use std::slice::Iter;
use std::iter::{Map, Zip};

#[derive(Debug)]
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

pub enum RelationColumnIter<'a, TItem> {
    FromSliceIter { v_iter : Iter<'a, TItem> }
}

impl<'a, TItem> Clone for RelationColumnIter<'a, TItem> {

    fn clone(&self) -> Self {

        match self {
            RelationColumnIter::FromSliceIter { v_iter } =>
                RelationColumnIter::FromSliceIter { v_iter: v_iter.clone() }
            
        }
    }
}

impl<'a,TItem> Iterator for RelationColumnIter<'a, TItem> {
    type Item = &'a TItem;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RelationColumnIter::FromSliceIter { v_iter } => v_iter.next()
        }
    }
}

pub enum RelationColumnDataRef<'a> {
    VecRef { vec_ref : &'a dyn Any }
}

impl <'a> RelationColumnDataRef<'a> {

    fn iter_vec<TItem>(
        column : &String,
        vec : &'a dyn Any
    ) -> RelationResult<RelationColumnIter<'a, TItem>>
        where TItem : Any {

        vec.downcast_ref::<Vec<TItem>>()
            .map(|vec| { RelationColumnIter::FromSliceIter { v_iter: vec.iter() }})
            .ok_or(RelationError::incorrect_column_type::<TItem>(column.clone()))
    }

    pub fn iter_as<TItem>(&self, column: &String) -> RelationResult<RelationColumnIter<'a, TItem>>
        where TItem : Any {

        match self {
            RelationColumnDataRef::VecRef { vec_ref } =>
                RelationColumnDataRef::iter_vec(column, *vec_ref)
        }
    }
}

///
pub struct RelationColumnRange<'a> {
    pub column_name : &'a String,
    column_values : RelationColumnDataRef<'a>
}

impl<'a> RelationColumnRange<'a> {

    fn iter_as<TItem>(&self) -> RelationResult<RelationColumnIter<'a, TItem>>
        where TItem : Any {

        return self.column_values.iter_as::<TItem>(self.column_name);
    }

    pub fn from_vector_ref<'t>(column : &'t String, vector_ref: &'t dyn Any) -> RelationColumnRange<'t> {
        return RelationColumnRange {
            column_name : column,
            column_values : RelationColumnDataRef::VecRef { vec_ref: vector_ref }
        }
    }
}

type DynamicArgs<'a> = Vec<RelationColumnRange<'a>>;

pub trait ToArgs : Sized {

    type Item<'a>;
    type Iter<'a> : Iterator<Item = Self::Item<'a>>;
    
    fn to_args<'a>(
        args : &'a Vec<RelationColumnRange<'a>>,
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

pub trait SelectDispatchFn<TResult> {

    fn dispatch<'a>(
        &self,
        args: &'a DynamicArgs<'a>
    ) -> RelationResult<SelectResult<TResult>>;
}

impl<A1> ToArgs for (&'_ A1,) where A1 : Any {

    type Item<'a> = (&'a A1,);
    type Iter<'a> = Map<RelationColumnIter<'a, A1>, fn(&A1) -> (&A1,)>;

    fn to_args<'a>(
        args : &'a DynamicArgs<'a>
    ) -> RelationResult<Self::Iter<'a>> {

        if let [column] = &args[0..] {
            return column.iter_as::<A1>().map(|it| {
                let mapper : fn(&A1) -> (&A1,) = |v| { (v,)};
                return it.map(mapper);
            });
        }
        else {
            return RelationError::raise_incorrect_column_count(1,args.len())
        }
    }
}

impl<A1,A2> ToArgs for (&'_ A1,&'_ A2)
    where
        A1 : Any,
        A2 : Any
    {

    type Item<'a> = (&'a A1, &'a A2);
    type Iter<'a> = Zip<RelationColumnIter<'a, A1>, RelationColumnIter<'a, A2>>;

    fn to_args<'a>(
        args : &'a DynamicArgs<'a>,
    ) -> RelationResult<Self::Iter<'a>> {

        if let [arg1, arg2] = &args[0..] {
            let it1 = arg1.iter_as::<A1>();
            let it2 = arg2.iter_as::<A2>();

            return it1.and_then(|a1| { 
                it2.map(|a2|{
                    a1.zip(a2)
                })
            });
        }
        else {
            return RelationError::raise_incorrect_column_count(2, args.len())
        }
    }
}

impl<A1, A2, TResult> SelectDispatchFn<TResult> for fn(&A1, &A2) -> TResult
    where
        A1 : Any,
        A2 : Any {
    
    fn dispatch<'a>(
        &self,
        args: &'a DynamicArgs<'a>
    ) -> RelationResult<SelectResult<TResult>> {
        
        let to_args_result = <(&A1, &A2) as ToArgs>::to_args(args);
        return to_args_result
            .map(|args| {
                args.map(|(a1, a2)| { self(a1, a2) }).collect()
            }
        )
    }
}

pub trait Relation {

    fn iter_range<TRange>(&self, column : &String, range : TRange) -> RelationColumnRange<'_>
        where TRange : RangeBounds<dyn Any>;

    fn try_select<F, TResult>(
        &self,
        columns : &Vec<String>,
        select : F
    ) -> RelationResult<SelectResult<TResult>>
    where F : SelectDispatchFn<TResult> {

        let args : Vec<_> = columns.iter().map(
            |column| { self.iter_range(column, RangeFull ) }
        ).collect();

        return select.dispatch(&args);
    }
}