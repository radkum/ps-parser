mod arithmetic;
mod comparison;
mod contain;
mod join;
mod replace;
mod split;
mod type_check;

pub(crate) use arithmetic::ArithmeticPred;
pub(crate) use comparison::ComparisonPred;
pub(crate) use replace::ReplacePred;
pub(crate) use type_check::TypeCheckPred;

use super::Val;

pub(crate) enum PredType {
    ArithmeticPred,
    ComparisonPred,
    ContainPred,
    JoinPred,
    ReplacePred,
    SplitPred,
    TypeCheckPred,
}

struct Predicate;

impl Predicate {}
