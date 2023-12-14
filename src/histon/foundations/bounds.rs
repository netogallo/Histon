use std::ops::Bound;

pub fn try_map<'a, F, T, U, E>(bound : &'a Bound<T>, f : F) -> Result<Bound<U>, E>
    where F : Fn(&'a T) -> Result<U,E> {

    match bound {
        Bound::Excluded(v) => f(v).map(Bound::Excluded),
        Bound::Included(v) => f(v).map(Bound::Included),
        Bound::Unbounded => Ok(Bound::Unbounded)
    }
}