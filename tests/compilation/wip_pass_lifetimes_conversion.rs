#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

// Conversion test: owned output, borrowed errors
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultConversion<'e, T> {
    Ok(T),
    Err(&'e str),
}

// Implement conversion from MyResult residual to a standard error type
impl From<MyResultConversion<'static, !>> for String {
    fn from(_: MyResultConversion<'static, !>) -> Self {
        "converted error".to_string()
    }
}

// Test converting from Result with borrowed error to MyResult
fn convert_result_borrowed_error_to_myresult<'e, T>(ok_val: T, err_val: &'e str) -> MyResultConversion<'e, T> {
    // This should work if Try_ConvertResult is implemented correctly
    let result: Result<(), &'e str> = Err(err_val);
    result?;
    MyResultConversion::Ok(ok_val)
}

// Test converting from MyResult to Result with matching lifetimes
fn convert_myresult_to_result<'e, T>(ok_val: T, err_val: &'e str) -> Result<T, String> {
    let my_result = MyResultConversion::Ok(ok_val)?;
    Ok(my_result)
}

// Test short-circuit conversion from Result to MyResult
fn short_circuit_conversion_result_to_myresult<'e, T>(ok_val: T, err_val: &'e str) -> MyResultConversion<'e, T> {
    let result: Result<(), &'e str> = Err(err_val);
    result?; // Should convert and short-circuit
    MyResultConversion::Ok(ok_val)
}

// Test lifetime preservation through conversion chain
fn conversion_chain_lifetimes<'e>(ok_val: i32, err_val: &'e str) -> Result<i32, String> {
    // Convert from Result to MyResult via ?
    let result: Result<(), &'e str> = Err(err_val);
    result?; // Should convert and short-circuit
    Ok(ok_val)
}
    result?; // Should convert and short-circuit
    Ok(ok_val)
}

fn main() {
    let ok_val = 42;
    let err_val = "error";

    // Test conversions
    let result1 = convert_result_borrowed_error_to_myresult(ok_val, &err_val);
    let result2 = convert_myresult_to_result(ok_val, &err_val);
    let result3 = short_circuit_conversion_result_to_myresult(ok_val, &err_val);
    let result4 = conversion_chain_lifetimes(ok_val, &err_val);

    // Verify results
    match result1 {
        MyResultConversion::Ok(val) => assert_eq!(val, 42),
        MyResultConversion::Err(_) => panic!("Should not short-circuit"),
    }

    match result2 {
        Ok(val) => assert_eq!(val, 42),
        Err(_) => panic!("Should not short-circuit"),
    }
}