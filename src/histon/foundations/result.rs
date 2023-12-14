use std::any::{Any, TypeId};

#[derive(Debug)]
pub enum RelationError {
    IncorrectColumnType { column : String, type_id : TypeId },
    IncorrectColumnCount { expected : usize, actual : usize },
    IncorrectBoundsType { expected: TypeId, actual : TypeId }
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

    pub fn incorrect_bounds_type<T>(
        actual : &dyn Any
    ) -> RelationError
        where T : Any {
        return RelationError::IncorrectBoundsType {
            expected: TypeId::of::<T>(), actual: actual.type_id()
        }
    }
}
