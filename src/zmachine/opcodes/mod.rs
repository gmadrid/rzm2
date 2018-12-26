mod branch;
mod zoperand;
mod zvariable;

use super::result::{Result, ZErr};
use super::traits::{PC, Variables};

pub use self::zoperand::{ZOperand, ZOperandType};
pub use self::zvariable::ZVariable;

//pub mod zero_op {
//
//}
//
//pub mod one_op {
//    use super::branch::jz;
//}
//
//pub mod two_op {
//
//}
//
//pub mod var_op {
//
//}
