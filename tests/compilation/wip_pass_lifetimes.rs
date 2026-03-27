#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

// Basic lifetime test: owned output, borrowed errors
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultLifetimeErrors<'e, T> {
    Ok(T),
    Err(&'e str),
}

// Test basic short-circuit with borrowed error
fn basic_short_circuit_borrowed_error<'e, T>(ok_val: T, err_val: &'e str) -> MyResultLifetimeErrors<'e, T> {
    let _ = MyResultLifetimeErrors::Err(err_val)?;
    MyResultLifetimeErrors::Ok(ok_val)
}

// Test lifetime elision with borrowed errors
fn lifetime_elision_borrowed_errors<T>(ok_val: T, err_val: &str) -> MyResultLifetimeErrors<'_, T> {
    let _ = MyResultLifetimeErrors::Err(err_val)?;
    MyResultLifetimeErrors::Ok(ok_val)
}

// Test multiple short-circuit points with same lifetime
fn multiple_short_circuits_same_lifetime<'e, T>(ok_val: T, err1_val: &'e str, err2_val: &'e str) -> MyResultLifetimeErrors<'e, T> {
    let _ = MyResultLifetimeErrors::Err(err1_val)?;
    let _ = MyResultLifetimeErrors::Err(err2_val)?;
    MyResultLifetimeErrors::Ok(ok_val)
}

fn main() {
    let ok_val = 42;
    let err_val = "error";

    // Test basic functionality
    let result1 = basic_short_circuit_borrowed_error(ok_val, &err_val);
    let _result2 = lifetime_elision_borrowed_errors(ok_val, &err_val);
    let _result3 = multiple_short_circuits_same_lifetime(ok_val, &err_val, &err_val);

    // Ensure results are used to avoid warnings
    match result1 {
        MyResultLifetimeErrors::Ok(val) => assert_eq!(val, 42),
        MyResultLifetimeErrors::Err(_) => panic!("Should not short-circuit"),
    }
}
