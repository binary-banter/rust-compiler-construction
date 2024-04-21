#[cfg(test)]
use derive_name::VariantName;
use miette::Diagnostic;
use thiserror::Error;

#[cfg_attr(test, derive(VariantName))]
#[derive(Debug, Error, Diagnostic)]
pub enum TypeError {
    #[error("Encountered an undeclared variable.")]
    UndeclaredVar {
        sym: String,
        #[label = "This variable `{sym}` was not declared yet"]
        span: (usize, usize),
    },
    #[error("Duplicate global definition.")]
    DuplicateGlobal {
        #[label = "Global `{sym}` was first declared here"]
        span1: (usize, usize),
        #[label = "And was redeclared here"]
        span2: (usize, usize),
        sym: String,
    },
    #[error("Duplicate argument name.")]
    DuplicateArg {
        #[label = "Argument `{sym}` was first declared here"]
        span1: (usize, usize),
        #[label = "And was redeclared here"]
        span2: (usize, usize),
        sym: String,
    },
    #[error("Break outside loop.")]
    BreakOutsideLoop {
        #[label = "Found a break outside of a loop"]
        span: (usize, usize),
    },
    #[error("Continue outside loop.")]
    ContinueOutsideLoop {
        #[label = "Found a continue outside of a loop"]
        span: (usize, usize),
    },
    #[error("Tried to modify immutable variable.")]
    ModifyImmutable {
        #[label = "This variable was declared as immutable."]
        span: (usize, usize),
    },
    #[error("Tried to put type in variable.'")]
    SymbolShouldBeVariable {
        #[label = "This should be a variable."]
        span: (usize, usize),
    },
    #[error("Tried to use variable as type.'")]
    SymbolShouldBeStruct {
        #[label = "This should be a struct."]
        span: (usize, usize),
    },
    #[error("Tried to use variable as type.'")]
    SymbolShouldBeEnum {
        #[label = "This should be an enum."]
        span: (usize, usize),
    },
    #[error("Unknown struct field.")]
    UnknownStructField {
        sym: String,
        #[label = "The field `{sym}` is not present in the struct definition."]
        span: (usize, usize),
    },
    #[error("Unknown enum variant.")]
    UnknownEnumVariant {
        sym: String,
        #[label = "The variant `{sym}` is not present in the enum definition."]
        span: (usize, usize),
    },
    #[error("Missing struct field.")]
    ConstructMissingField {
        sym: String,
        #[label = "The field `{sym}` is missing in the struct."]
        struct_span: (usize, usize),
        #[label = "It was defined here."]
        def_span: (usize, usize),
    },
    #[error("Duplicate struct field.")]
    ConstructDuplicateField {
        sym: String,
        #[label = "The field `{sym}` was already provided earlier."]
        span: (usize, usize),
    },
    #[error("Unsized type.")]
    UnsizedType {
        sym: String,
        #[label = "The type definition `{sym}` is not sized."]
        span: (usize, usize),
    },
    #[error("Could not parse integer.")]
    InvalidInteger {
        #[label = "`{val}` is not a valid {typ}: {err}"]
        span: (usize, usize),
        val: String,
        typ: &'static str,
        err: String,
    },
    #[error("Could not parse integer.")]
    InvalidByteLit {
        #[label = "`{val}` is not a valid {typ}"]
        span: (usize, usize),
        val: String,
        typ: &'static str,
    },
    #[error("Could not parse integer.")]
    InvalidEscape {
        #[label = "`{val}` is not an escape sequence (or has not been added to the compiler as an escape sequence!)"]
        span: (usize, usize),
        val: String,
    },
    #[error("Integer ambiguous.")]
    IntegerAmbiguous {
        #[label = "Could not determine the exact type of this integer"]
        span: (usize, usize),
    },
    #[error("The program doesn't have a main function.")]
    NoMain,
    #[error("Types did not match.")]
    MismatchedFnReturn {
        expect: String,
        got: String,

        //TODO would like this span to be return type if present
        #[label = "Expected this function to return: `{expect}`"]
        span_expected: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_got: (usize, usize),
    },
    #[error("Types did not match.")]
    OperandExpect {
        expect: String,
        got: String,
        op: String,

        //TODO would like this span to be operator
        #[label = "Arguments of {op} are of type: `{expect}`"]
        span_op: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_arg: (usize, usize),
    },
    #[error("Types did not match.")]
    OperandEqual {
        lhs: String,
        rhs: String,
        op: String,

        //TODO would like this span to be operator
        #[label = "Arguments of {op} should be of equal types"]
        span_op: (usize, usize),
        #[label = "Type: `{lhs}`"]
        span_lhs: (usize, usize),
        #[label = "Type: `{rhs}`"]
        span_rhs: (usize, usize),
    },
    #[error("Types did not match.")]
    MismatchedLetBinding {
        got: String,

        //TODO would like this span of type of let binding
        #[label = "Expected binding of let to have this type"]
        span_expected: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_got: (usize, usize),
    },
    #[error("Types did not match.")]
    MismatchedAssignBinding {
        expect: String,
        got: String,

        #[label = "Expected binding of assign to have type: `{expect}`"]
        span_expected: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_got: (usize, usize),
    },
    #[error("Types did not match.")]
    MismatchedStructField {
        expect: String,
        got: String,

        #[label = "Expected struct field to have type: `{expect}`"]
        span_expected: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_got: (usize, usize),
    },
    #[error("Types did not match.")]
    MismatchedEnumVariant {
        expect: String,
        got: String,

        #[label = "Expected enum variant to have type: `{expect}`"]
        span_expected: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_got: (usize, usize),
    },
    #[error("Types did not match.")]
    IfExpectBool {
        got: String,

        #[label = "Expected this condition to be `Bool`, but got: `{got}`"]
        span_got: (usize, usize),
    },

    #[error("Types did not match.")]
    IfExpectEqual {
        thn: String,
        els: String,

        #[label = "Type: `{thn}`"]
        span_thn: (usize, usize),
        #[label = "Type: `{els}`"]
        span_els: (usize, usize),
    },

    #[error("Types did not match.")]
    MismatchedExpectFn {
        got: String,

        #[label = "Expected function, but found '{got}'"]
        span_got: (usize, usize),
    },

    #[error("Types did not match.")]
    MismatchedLoop {
        expect: String,
        got: String,

        #[label = "Expected loop to return `{expect}`, but got: `{got}``"]
        span_break: (usize, usize),
    },

    #[error("Types did not match.")]
    ArgCountMismatch {
        expected: usize,
        got: usize,

        //TODO span of args
        #[label = "Function {expected} arguments, but found {got} arguments"]
        span: (usize, usize),
    },

    #[error("Types did not match.")]
    FnArgExpect {
        param: String,
        arg: String,

        #[label = "Expected this function argument to be of type `{param}`, but got `{arg}`"]
        span_arg: (usize, usize),
    },
}
