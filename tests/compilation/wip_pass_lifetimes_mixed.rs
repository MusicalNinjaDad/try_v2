#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

// Mixed ownership test: owned output, borrowed errors
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultOwnedOutputBorrowedErrors<'e, T> {
    Ok(T),
    Err(&'e str),
}

// Test owned output with borrowed errors
fn owned_output_borrowed_errors<'e, T>(ok_val: T, err_val: &'e str) -> MyResultOwnedOutputBorrowedErrors<'e, T> {
    let result = MyResultOwnedOutputBorrowedErrors::Ok(ok_val)?;
    MyResultOwnedOutputBorrowedErrors::Ok(result)
}

// Test short-circuit with borrowed error
fn short_circuit_borrowed_error_owned_output<'e, T>(ok_val: T, err_val: &'e str) -> MyResultOwnedOutputBorrowedErrors<'e, T> {
    let _ = MyResultOwnedOutputBorrowedErrors::Err(err_val)?;
    MyResultOwnedOutputBorrowedErrors::Ok(ok_val)
}

// Test that owned values work after short-circuit
fn return_owned_after_short_circuit(ok_val: i32, err_val: String) -> i32 {
    let result = MyResultOwnedBoth::Ok(ok_val)?;
    result
}

// Mixed ownership test: owned output, owned errors (baseline)
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultOwnedBoth<T> {
    Ok(T),
    Err(String),
}

fn main() {
    let ok_val = 42;
    let err_str = "error";

    // Test owned output, borrowed errors
    let result1 = owned_output_borrowed_errors(ok_val, &err_str);
    let result2 = short_circuit_borrowed_error_owned_output(ok_val, &err_str);

    // Test that owned values work after short-circuit
    let result3 = return_owned_after_short_circuit(ok_val, "error".to_string());

    // Verify results
    match result1 {
        MyResultOwnedOutputBorrowedErrors::Ok(val) => assert_eq!(val, 42),
        MyResultOwnedOutputBorrowedErrors::Err(_) => panic!("Should not short-circuit"),
    }

    match result2 {
        MyResultOwnedOutputBorrowedErrors::Ok(_) => panic!("Should have short-circuited"),
        MyResultOwnedOutputBorrowedErrors::Err(e) => assert_eq!(*e, "error"),
    }

    assert_eq!(result3, 42);
}

    match result1 {
        MyResultOwnedOutputBorrowedErrors::Ok(val) => assert_eq!(val, 42),
        MyResultOwnedOutputBorrowedErrors::Err(_) => panic!("Should not short-circuit"),
    }

    match result3 {
        MyResultOwnedBoth::Ok(val) => assert_eq!(val, 42),
        MyResultOwnedBoth::Err(_) => panic!("Should not short-circuit"),
    }
}