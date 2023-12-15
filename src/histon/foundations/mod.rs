pub mod bounds;
pub mod column;
pub mod control;
pub mod selection;
pub mod result;

use std::any::Any;
use std::iter::{Map, Zip};

pub use self::column::*;
use self::control::LiftErr;
use self::result::*;
use self::selection::*;

use super::support::ToColumn;

impl<TItem> RelationColumnSelection<TItem> {

    pub fn try_map<'a, F, TOut, TErr>(&'a self, f : F) -> Result<RelationColumnSelection<TOut>, TErr>
        where F : Fn(&'a TItem) -> Result<TOut,TErr> {

        match self {
            Self::BoundsSelection { bounds: (start, end) } => {
                let b1 = bounds::try_map(start, &f);
                let b2 = bounds::try_map(end, &f);
                (b1,b2).lift_err(|b1, b2| {
                    Ok(RelationColumnSelection::BoundsSelection { bounds: (b1, b2) })
                })
            },
            Self::AllItems => Ok(RelationColumnSelection::AllItems)
        }
    }
}

impl RelationColumnSelection<Box<dyn Any>> {

    pub fn from_any<TValue>(&self) -> RelationResult<RelationColumnSelection<&'_ TValue>>
        where TValue : Any {

        self.try_map(|item| {
            item.downcast_ref::<TValue>().ok_or(
                RelationError::incorrect_bounds_type::<TValue>(item.as_ref())
            )
        })
    }
}
pub trait ToArgs : Sized {

    type Item<'a>;
    type Iter<'a> : Iterator<Item = Self::Item<'a>>;
    
    fn to_args<'t, TColumn>(
        args : &'t Vec<TColumn>,
    ) -> RelationResult<Self::Iter<'t>>
    where
        TColumn : RelationColumnRange;
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

    fn dispatch<'a, TColumn>(
        &self,
        args: &'a Vec<TColumn>
    ) -> RelationResult<SelectResult<TResult>>
    where
        TColumn : RelationColumnRange;
}

impl<A1> ToArgs for (&'_ A1,) where A1 : Any + Ord {

    type Item<'a> = (&'a A1,);
    type Iter<'a> = Map<RelationColumnIter<'a, A1>, fn(&A1) -> (&A1,)>;

    fn to_args<'t, TColumn>(
        args : &'t Vec<TColumn>
    ) -> RelationResult<Self::Iter<'t>>
    where
        TColumn : RelationColumnRange {

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
        A1 : Any + Ord,
        A2 : Any + Ord
    {

    type Item<'a> = (&'a A1, &'a A2);
    type Iter<'a> = Zip<RelationColumnIter<'a, A1>, RelationColumnIter<'a, A2>>;

    fn to_args<'t, TColumn>(
        args : &'t Vec<TColumn>,
    ) -> RelationResult<Self::Iter<'t>>
    where
        TColumn : RelationColumnRange {

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
        A1 : Any + Ord,
        A2 : Any + Ord {
    
    fn dispatch<'a, TColumn>(
        &self,
        args: &'a Vec<TColumn>
    ) -> RelationResult<SelectResult<TResult>>
    where
        TColumn : RelationColumnRange {
        
        let to_args_result = <(&A1, &A2) as ToArgs>::to_args(args);
        return to_args_result
            .map(|args| {
                args.map(|(a1, a2)| { self(a1, a2) }).collect()
            }
        )
    }
}

pub trait Relation<'t> {

    type RelationColumn : RelationColumnRange + 't;

    fn iter_selection(
        &'t self, column : &String,
        range : Option<RelationColumnSelectionDyn>
    ) -> Self::RelationColumn;

    fn try_select<F, TResult>(
        &'t self,
        columns : &Vec<String>,
        select : F
    ) -> RelationResult<SelectResult<TResult>>
    where F : SelectDispatchFn<TResult> {

        let args : Vec<_> = columns.iter().map(
            |column| { self.iter_selection(column, None) }
        ).collect();

        return select.dispatch(&args);
    }
}