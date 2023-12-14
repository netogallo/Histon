mod bounds;
mod control;
mod result;

use std::any::Any;
use std::ops::{RangeBounds, RangeFull, Bound};
use std::slice::Iter;
use std::iter::{Map, Zip};

use self::control::LiftErr;
use self::result::*;

pub enum RelationColumnContainer<'a, TItem> {
    FromSliceIter { v_iter : Iter<'a, TItem> }
}

impl<'a, TItem> Clone for RelationColumnContainer<'a, TItem> {

    fn clone(&self) -> Self {

        match self {
            RelationColumnContainer::FromSliceIter { v_iter } =>
            RelationColumnContainer::FromSliceIter { v_iter: v_iter.clone() }
            
        }
    }
}

impl<'a,TItem> Iterator for RelationColumnContainer<'a, TItem> {
    type Item = &'a TItem;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RelationColumnContainer::FromSliceIter { v_iter } => v_iter.next()
        }
    }
}

#[derive(Clone)]
pub enum RelationColumnSelection<TItem> {
    BoundsSelection { bounds : (Bound<TItem>, Bound<TItem>) },
    AllItems
}

pub type RelationColumnSelectionDyn = RelationColumnSelection<Box<dyn Any>>;

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

impl<TItem> RelationColumnSelection<TItem> {
    
    fn contains(&self, other: &TItem) -> bool
        where TItem : Ord {

        match self {
            Self::BoundsSelection { bounds } =>
                bounds.contains(other),
            Self::AllItems => true
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

pub struct RelationColumnIter<'a, TItem> {
    container: RelationColumnContainer<'a, TItem>,
    selection: RelationColumnSelection<&'a TItem>
}

impl<'a, TItem> Clone for RelationColumnIter<'a, TItem> {
    
    fn clone(&self) -> Self {
        return RelationColumnIter {
            container: self.container.clone(),
            selection: self.selection.clone()
        }
    }
}

impl<'a,TItem> Iterator for RelationColumnIter<'a, TItem>
    where TItem : Ord {
    type Item = &'a TItem;

    fn next(&mut self) -> Option<Self::Item> {

        while let Some(next) = self.container.next() {

            if self.selection.contains(&next) {
                return Some(next)
            }
        }

        return None
    }
}

pub enum RelationColumnDataRef<'a> {
    VecRef { vec_ref : &'a dyn Any }
}

impl <'a> RelationColumnDataRef<'a> {

    fn iter_vec<TItem>(
        column : &String,
        bounds : &'a RelationColumnSelectionDyn,
        vec : &'a dyn Any
    ) -> RelationResult<RelationColumnIter<'a, TItem>>
        where TItem : Any {

        let container = vec.downcast_ref::<Vec<TItem>>()
            .map(|vec| { RelationColumnContainer::FromSliceIter { v_iter: vec.iter() }})
            .ok_or(RelationError::incorrect_column_type::<TItem>(column.clone()));

        let selection = bounds.from_any::<TItem>();

        (selection, container).lift_err(|ok_selection, ok_container| {
            Ok(
                RelationColumnIter {
                    container: ok_container,
                    selection: ok_selection
                }
            )
        })
    }

    pub fn iter_as<TItem>(
        &self,
        column: &String,
        bounds : &'a RelationColumnSelectionDyn
    ) -> RelationResult<RelationColumnIter<'a, TItem>>
        where TItem : Any {

        match self {
            RelationColumnDataRef::VecRef { vec_ref } =>
                RelationColumnDataRef::iter_vec(column, bounds,*vec_ref)
        }
    }
}

///
pub struct RelationColumnRange<'a> {
    pub column_name : &'a String,
    column_values : RelationColumnDataRef<'a>,
    range : RelationColumnSelectionDyn
}

impl<'a> RelationColumnRange<'a> {

    fn iter_as<TItem>(&'a self) -> RelationResult<RelationColumnIter<'a, TItem>>
        where TItem : Any {

        self.column_values.iter_as::<TItem>(self.column_name, &self.range)
    }

    pub fn from_vector_ref<'t>(
        column : &'t String,
        vector_ref: &'t dyn Any,
        range : Option<RelationColumnSelectionDyn>
    ) -> RelationColumnRange<'t> {

        return RelationColumnRange {
            column_name : column,
            column_values : RelationColumnDataRef::VecRef { vec_ref: vector_ref },
            range : range.unwrap_or(RelationColumnSelection::AllItems)
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

impl<A1> ToArgs for (&'_ A1,) where A1 : Any + Ord {

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
        A1 : Any + Ord,
        A2 : Any + Ord
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
        A1 : Any + Ord,
        A2 : Any + Ord {
    
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

    fn iter_selection(
        &self, column : &String,
        range : Option<RelationColumnSelectionDyn>
    ) -> RelationColumnRange<'_>;

    fn try_select<F, TResult>(
        &self,
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