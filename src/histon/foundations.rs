use std::any::{Any, TypeId};
use std::rc::Rc;

enum RelationError {
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
        column : String,
    ) -> RelationResult<T>
    where T : Any {
        return Result::Err(RelationError::IncorrectColumnType { column, type_id: TypeId::of::<T>() })
    }

    pub fn incorrect_column_type<T>(
        column : String,
    ) -> RelationError
    where T : Any {
        return RelationError::IncorrectColumnType { column, type_id: TypeId::of::<T>() }
    }
}

struct ToArgsIterator<'a, TValue> {
    values : &'a Vec<TValue>
}

impl<'a, TValue> Iterator for ToArgsIterator<'a, &TValue> {
    type Item = TValue;

    fn next(&mut self) -> Option<Self::Item> {
        panic!("not implemented")
    }
}

impl<'a, TValue> ToArgsIterator<'a, TValue> {

    pub fn from_any_ref(
        column : String,
        the_box : & dyn Any
    ) -> RelationResult<Rc<Self>>
    where Self : 'static {

        let self_case = the_box.downcast_ref::<Self>();
        let vec_case = the_box.downcast_ref::<Vec<TValue>>().map(
            |vec| { ToArgsIterator { values : vec } }
        );

        return self_case.or(vec_case)
            .map_err(|_box| { RelationError::incorrect_column_type::<Box<Self>>(column) });
    }
}


pub trait ToArgs : Any + Sized {
    
    fn to_args (
        columns : &Vec<String>,
        args : &Vec<Rc<dyn Any>>
    ) -> Result<ToArgsIterator<Self>, RelationError>;
}

pub struct SelectIterator<Values> {
    pub values : Vec<Values>
}

impl<TValue> FromIterator<TValue> for SelectIterator<TValue> {

    fn from_iter<T: IntoIterator<Item = TValue>>(iter: T) -> Self {
        panic!("not implemented")
    }
}

pub trait SelectDispatchFn<Args, TResult> {

    fn dispatch(
        &self,
        columns : &Vec<String>,
        args: &Vec<Rc<dyn Any>>
    ) -> RelationResult<SelectIterator<TResult>>;
}

impl<A1> ToArgs for (&'static A1,)
    where
        A1 : Any
    {

    fn to_args(
        columns : &Vec<String>,
        args : &Vec<Rc<dyn Any>>) -> RelationResult<ToArgsIterator<(&'static A1,)>> {

        if args.len() != 1 {
            return RelationError::raise_incorrect_column_count(1,columns.len())
        }

        return ToArgsIterator::from_any_ref(columns[0], args[0])
            .map(|v| {
                v.map(|v| { (v,) }).collect()
            })
            .or_else(|_e| RelationError::raise_incorrect_column_type(columns[0]));
    }
}

impl<A1,A2> ToArgs for (&'static A1,&'static A2)
    where
        A1 : Any + Clone
        , A2 : Any + Clone {

    fn to_args(
        columns : &Vec<String>,
        args : &Vec<Rc<dyn Any>>
    ) -> RelationResult<ToArgsIterator<(&'static A1,&'static A2)>> {

        if args.len() != 2 {
            return RelationError::raise_incorrect_column_count(2, columns.len())
        }

        let arg1 : RelationResult<Rc<ToArgsIterator<&A1>>> =
            ToArgsIterator::from_any_ref(columns[0], args[0])
            .map_err(|_e| { RelationError::incorrect_column_type::<Box<A1>>(columns[0]) });
        
        let arg2 : RelationResult<Rc<ToArgsIterator<&A2>>> =
            ToArgsIterator::from_any_ref(columns[1], args[1])
            .map_err(|_e| { RelationError::incorrect_column_type::<Box<A2>>(columns[1])});

        return arg1.and_then(|a1| { 
            arg2.map(|a2|{
                // a1.zip(a2).collect()
                panic!("no")
            })
        });
    }
}

impl<F, Args, TResult> SelectDispatchFn<Args, TResult> for F
    where
        F : FnMut(Args) -> TResult
        , Args : ToArgs {
    
    fn dispatch(
        &self,
        columns: &Vec<String>,
        args: &Vec<Rc<dyn Any>>
    ) -> RelationResult<SelectIterator<TResult>> {
        
        let args : RelationResult<ToArgsIterator<Args>> = ToArgs::to_args(columns, args);
        return args.map(|args| { args.map(|v| { self(v) }).collect() })
    }
}

pub trait Relation {
    fn try_select<F, Args, TResult>(
        &self,
        columns : &Vec<String>,
        select : F
    ) -> RelationResult<SelectIterator<TResult>>
    where F : SelectDispatchFn<Args, TResult>;
}