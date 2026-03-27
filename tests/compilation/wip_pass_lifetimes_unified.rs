#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

// Unified lifetime test: single lifetime for both output and errors
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultUnified<'r, T> {
    Ok(T),
    Err(&'r str),
}

// Test that unified lifetime works for owned output and borrowed errors
fn unified_lifetime_ok<'r, T>(ok_val: T, err_val: &'r str) -> MyResultUnified<'r, T> {
    let result = MyResultUnified::Ok(ok_val)?;
    MyResultUnified::Ok(result)
}

// Test short-circuit preserves unified lifetime
fn unified_lifetime_err<'r, T>(ok_val: T, err_val: &'r str) -> MyResultUnified<'r, T> {
    let _ = MyResultUnified::Err(err_val)?;
    MyResultUnified::Ok(ok_val)
}

// Test that values with same lifetime can be used together
fn unified_same_scope(ok_val: i32, err_val: &str) -> MyResultUnified {
    // Both references have the same implicit lifetime
    let _ = MyResultUnified::Err(err_val)?;
    let result = MyResultUnified::Ok(ok_val)?;
    MyResultUnified::Ok(result)
}

fn main() {
    let ok_val = 42;
    let err_val = "error";

    let result1 = unified_lifetime_ok(ok_val, &err_val);
    let result2 = unified_lifetime_err(ok_val, &err_val);
    let result3 = unified_same_scope(ok_val, &err_val);

    match result1 {
        MyResultUnified::Ok(val) => assert_eq!(val, 42),
        MyResultUnified::Err(_) => panic!("Should not short-circuit"),
    }
}