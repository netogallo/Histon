pub trait LiftErr {
    type TError;
    type TFn<TResult>;

    fn lift_err<TResult>(self, cont : Self::TFn<TResult>) -> Result<TResult, Self::TError>;
}

impl<A1, A2, TError> LiftErr for (Result<A1, TError>, Result<A2, TError>) {
    type TError = TError;
    type TFn<TResult> = fn(A1, A2) -> Result<TResult, Self::TError>;

    fn lift_err<TResult>(self, cont : Self::TFn<TResult>) -> Result<TResult, Self::TError> {

        return match self {
            (Ok(arg1), Ok(arg2)) => cont(arg1, arg2),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e)
        };
    }
}