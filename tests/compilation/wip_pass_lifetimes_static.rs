#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

// Static lifetime test: errors have static lifetime bound
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultStaticErrors<T> {
    Ok(T),
    Err(&'static str),
}

// Test owned output with static error references
fn owned_output_static_errors<T>(ok_val: T, err_val: &'static str) -> MyResultStaticErrors<T> {
    let result = MyResultStaticErrors::Ok(ok_val)?;
    MyResultStaticErrors::Ok(result)
}

// Test short-circuit with static error reference
fn short_circuit_static_errors<T>(ok_val: T, err_val: &'static str) -> MyResultStaticErrors<T> {
    let _ = MyResultStaticErrors::Err(err_val)?;
    MyResultStaticErrors::Ok(ok_val)
}

// Test with both static bounds (owned types)
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultAllStatic<T> {
    Ok(T),
    Err(String),
}

// Test both static bounds with owned types
fn all_static_bounds(ok_val: i32, err_val: String) -> MyResultAllStatic {
    let result = MyResultAllStatic::Ok(ok_val)?;
    MyResultAllStatic::Ok(result)
}

// Test short-circuit with all static owned types
fn short_circuit_all_static(ok_val: i32, err_val: String) -> MyResultAllStatic {
    let _ = MyResultAllStatic::Err(err_val)?;
    MyResultAllStatic::Ok(ok_val)
}

// Test static lifetime with references (limited case)
fn static_references(ok_val: &'static i32, err_val: &'static str) -> MyResultStaticErrors<&'static i32> {
    let result = MyResultStaticErrors::Ok(ok_val)?;
    MyResultStaticErrors::Ok(result)
}

fn main() {
    // Static values for testing
    static OK_VAL: i32 = 42;
    static ERR_VAL: &str = "error";

    // Test static error cases
    let result1 = owned_output_static_errors(42, &ERR_VAL);
    let result2 = short_circuit_static_errors(42, &ERR_VAL);

    // Test all static cases
    let result3 = all_static_bounds(42, "error".to_string());
    let result4 = short_circuit_all_static(42, "error".to_string());

    // Test static references
    let result5 = static_references(&OK_VAL, &ERR_VAL);

    // Verify results
    match result1 {
        MyResultStaticErrors::Ok(val) => assert_eq!(val, 42),
        MyResultStaticErrors::Err(_) => panic!("Should not short-circuit"),
    }

    match result3 {
        MyResultAllStatic::Ok(val) => assert_eq!(val, 42),
        MyResultAllStatic::Err(_) => panic!("Should not short-circuit"),
    }

    match result5 {
        MyResultStaticErrors::Ok(val) => assert_eq!(*val, 42),
        MyResultStaticErrors::Err(_) => panic!("Should not short-circuit"),
    }
}