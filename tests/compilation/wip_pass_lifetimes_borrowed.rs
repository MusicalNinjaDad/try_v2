#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

// Borrowed output test: borrowed output with borrowed errors
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultBorrowedOutput<'o, 'e> {
    Ok(&'o i32),
    Err(&'e str),
}

// Test borrowed output with borrowed errors
fn borrowed_output_borrowed_errors<'o, 'e>(ok_val: &'o i32, err_val: &'e str) -> MyResultBorrowedOutput<'o, 'e> {
    let result = MyResultBorrowedOutput::Ok(ok_val)?;
    MyResultBorrowedOutput::Ok(result)
}

// Test short-circuit with borrowed output
fn short_circuit_borrowed_output<'o, 'e>(ok_val: &'o i32, err_val: &'e str) -> MyResultBorrowedOutput<'o, 'e> {
    let _ = MyResultBorrowedOutput::Err(err_val)?;
    MyResultBorrowedOutput::Ok(ok_val)
}

// Test lifetime variance with borrowed output
fn lifetime_variance_borrowed<'long: 'short, 'short>(ok_val: &'long i32, err_val: &'short str) -> MyResultBorrowedOutput<'long, 'short> {
    let result = MyResultBorrowedOutput::Ok(ok_val)?;
    MyResultBorrowedOutput::Ok(result)
}

fn main() {
    let ok_val = 42;
    let err_val = "error";

    let result1 = borrowed_output_borrowed_errors(&ok_val, &err_val);
    let result2 = short_circuit_borrowed_output(&ok_val, &err_val);
    let result3 = lifetime_variance_borrowed(&ok_val, &err_val);

    match result1 {
        MyResultBorrowedOutput::Ok(val) => assert_eq!(*val, 42),
        MyResultBorrowedOutput::Err(_) => panic!("Should not short-circuit"),
    }

    match result2 {
        MyResultBorrowedOutput::Ok(_) => panic!("Should have short-circuited"),
        MyResultBorrowedOutput::Err(e) => assert_eq!(*e, "error"),
    }

    match result3 {
        MyResultBorrowedOutput::Ok(val) => assert_eq!(*val, 42),
        MyResultBorrowedOutput::Err(_) => panic!("Should not short-circuit"),
    }
}