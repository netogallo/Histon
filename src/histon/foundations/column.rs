use std::any::Any;
use std::slice::Iter;

use super::control::LiftErr;
use super::result::*;
use super::selection::*;

pub enum RelationColumnDataRef<'a> {
    VecRef { vec_ref : &'a dyn Any }
}

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

            //next.eval_selection(self.selection);

            if self.selection.contains(&next) {
                return Some(next)
            }
        }

        return None
    }
}

impl <'a> RelationColumnDataRef<'a> {

    fn iter_vec<'t, TItem>(
        column : &String,
        bounds : &'t RelationColumnSelectionDyn,
        vec : &'a dyn Any
    ) -> RelationResult<RelationColumnIter<'t, TItem>>
        where
            TItem : Any,
            'a : 't {

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

    pub fn iter_as<'t, TItem>(
        &self,
        column: &'a String,
        bounds : &'t RelationColumnSelectionDyn
    ) -> RelationResult<RelationColumnIter<'t, TItem>>
        where
            TItem : Any,
            'a : 't {

        match self {
            RelationColumnDataRef::VecRef { vec_ref } =>
                RelationColumnDataRef::iter_vec(column, bounds,*vec_ref)
        }
    }
}

pub trait RelationColumnRange {
    fn iter_as<'t, TItem>(&'t self) -> RelationResult<RelationColumnIter<'t, TItem>>
        where
            TItem : Any;
}

///
pub struct RelationColumnLinearRange<'a> {
    pub column_name : &'a String,
    column_values : RelationColumnDataRef<'a>,
    range : RelationColumnSelectionDyn
}

impl RelationColumnLinearRange<'_> {
    pub fn from_vector_ref<'t>(
        column : &'t String,
        vector_ref: &'t dyn Any,
        range : Option<RelationColumnSelectionDyn>
    ) -> RelationColumnLinearRange<'t> {

        return RelationColumnLinearRange {
            column_name : column,
            column_values : RelationColumnDataRef::VecRef { vec_ref: vector_ref },
            range : range.unwrap_or(RelationColumnSelection::AllItems)
        }
    }
}

impl RelationColumnRange for RelationColumnLinearRange<'_> {

    fn iter_as<'t, TItem>(&'t self) -> RelationResult<RelationColumnIter<'t, TItem>>
        where
            TItem : Any {

        self.column_values.iter_as::<TItem>(self.column_name, &self.range)
    }
}