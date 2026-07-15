use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.character)
    }
}

#[derive(Debug, Error)]
pub enum MovaError {
    #[error("Lexer error at {position}: Unexpected character: '{character}'")]
    Lexer { character: char, position: Position },
    #[error("Parser error: {0}")]
    Parser(#[from] ParserError),
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Expected argument list to be closed")]
    ExpectedArgumentListToBeClosed,
    #[error("Expected comma or argument list to be closed")]
    ExpectedCommaOrArgumentListToBeClosed,
    #[error("Expected identifier to be called but found {0}")]
    ExpectedIdentifierToBeCalled(String),
    #[error("Expected ')' but found {0}")]
    ExpectedClosingParenthesis(String),
    #[error("Expected ')' but found end of input")]
    ExpectedClosingParenthesisButFoundEndOfInput,
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
    #[error("Unexpected token found: {0}")]
    UnexpectedToken(String),
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("Expected block to be closed")]
    ExpectedBlockToBeClosed,
    #[error("Expected identifier but got: {0}")]
    ExpectedIdentifierButGot(String),
    #[error("Expected identifier after `let` keyword")]
    ExpectedIdentifierAfterLet,
    #[error("Expected assignment after identifier")]
    ExpectedAssignmentAfterIdentifier,
    #[error("Expected function name after `fn` keyword")]
    ExpectedFunctionName,
    #[error("Expected parameter list after function name")]
    ExpectedParameterList,
    #[error("Expected parameter list to be closed")]
    ExpectedParameterListToBeClosed,
    #[error("Expected assignment before function body")]
    ExpectedAssignmentBeforeFunctionBody,
    #[error("Unexpected keyword found: {0}")]
    UnexpectedKeyword(String),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Unexpected operator '{operator}' for operands '{left}' and '{right}'")]
    UnexpectedOperator { operator: String, left: String, right: String },
    #[error("Expected {expected} arguments but received {received}")]
    InvalidArgumentCount { expected: usize, received: usize },
    #[error("Expected expression, but received statement as argument")]
    ExpectedExpressionAsArgument,
    #[error("'{0}' is not callable")]
    NotCallable(String),
    #[error("Expression cannot be referenced")]
    ExpressionCannotBeReferenced,
    #[error("Reference target yielded no value")]
    ReferenceTargetYieldedNoValue,
    #[error("Expected expression, but received statement as left operand")]
    ExpectedExpressionAsLeftOperand,
    #[error("Expected expression, but received statement as right operand")]
    ExpectedExpressionAsRightOperand,
    #[error("Dereference target yielded no value")]
    DereferenceTargetYieldedNoValue,
    #[error("Cannot read from moved value")]
    CannotReadFromMovedValue,
    #[error("Cannot dereference non-reference value")]
    CannotDereferenceNonReferenceValue,
    #[error("Expected expression, but received statement as value")]
    ExpectedExpressionAsValue,
    #[error("Cannot assign to deallocated variable '{0}'")]
    CannotAssignToDeallocatedVariable(String),
    #[error("Cannot assign to borrowed variable '{0}'")]
    CannotAssignToBorrowedVariable(String),
    #[error("Cannot assign to mutably borrowed variable '{0}'")]
    CannotAssignToMutablyBorrowedVariable(String),
    #[error("Cannot assign to immutable variable '{0}'")]
    CannotAssignToImmutableVariable(String),
    #[error("Assignment value yielded no value")]
    AssignmentValueYieldedNoValue,
    #[error("Condition yielded no value")]
    ConditionYieldedNoValue,
    #[error("Condition must be a boolean")]
    ConditionMustBeBoolean,
    #[error("Unable to resolve {0}")]
    UnableToResolve(String),
    #[error("Variable '{0}' already exists")]
    VariableAlreadyExists(String),
    #[error("Unable to use '{0}' because it is moved")]
    UnableToUseBecauseMoved(String),
    #[error("Unable to use '{0}' because it is deallocated")]
    UnableToUseBecauseDeallocated(String),
    #[error("Unable to mutate '{0}' because it is immutably borrowed")]
    UnableToMutateBecauseImmutablyBorrowed(String),
    #[error("Unable to mutate '{0}' because it is mutably borrowed")]
    UnableToMutateBecauseMutablyBorrowed(String),
    #[error("Unable to borrow value because it is moved")]
    UnableToBorrowBecauseMoved,
    #[error("Unable to borrow value because it is deallocated")]
    UnableToBorrowBecauseDeallocated,
    #[error("Unable to borrow because it is already mutably borrowed")]
    UnableToBorrowBecauseMutablyBorrowed,
    #[error("Unable to borrow mutably because it is already borrowed")]
    UnableToBorrowMutablyBecauseBorrowed,
    #[error("Accessing a deallocated reference")]
    AccessingDeallocatedReference,
    #[error("Assigning to a deallocated reference")]
    AssigningToDeallocatedReference,
    #[error("Cannot assign to an immutable reference")]
    CannotAssignToImmutableReference,
}

pub type Result<T> = std::result::Result<T, MovaError>;
