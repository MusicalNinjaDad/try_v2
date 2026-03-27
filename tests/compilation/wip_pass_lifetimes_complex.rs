#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

// Complex lifetime test: multiple lifetime parameters with different usage
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultComplex<'a, 'b, T> {
    Ok(T),
    Err1(&'a str),
    Err2(&'b str),
}

// Test multiple error variants with different lifetimes
fn multiple_error_lifetimes<'a, 'b, T>(ok_val: T, err1_val: &'a str, err2_val: &'b str) -> MyResultComplex<'a, 'b, T> {
    let result = MyResultComplex::Ok(ok_val)?;
    MyResultComplex::Ok(result)
}

// Test short-circuit through different error variants
fn short_circuit_different_errors<'a, 'b, T>(ok_val: T, err1_val: &'a str, err2_val: &'b str) -> MyResultComplex<'a, 'b, T> {
    let _ = MyResultComplex::Err1(err1_val)?;
    MyResultComplex::Ok(ok_val)
}

// Test the other error variant
fn short_circuit_other_error<'a, 'b, T>(ok_val: T, err1_val: &'a str, err2_val: &'b str) -> MyResultComplex<'a, 'b, T> {
    let _ = MyResultComplex::Err2(err2_val)?;
    MyResultComplex::Ok(ok_val)
}

// Nested lifetime test: MyResult containing another type with lifetimes
#[derive(Debug)]
struct Wrapper<'w> {
    value: &'w i32,
}

#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultNested<'r, 'w, T> {
    Ok(&'r Wrapper<'w>),
    Err(&'r str),
}

// Test nested lifetimes
fn nested_lifetimes<'r, 'w, T>(wrapper: &'r Wrapper<'w>, err_val: &'r str) -> MyResultNested<'r, 'w, T> {
    let result = MyResultNested::Ok(wrapper)?;
    MyResultNested::Ok(result)
}

// Test short-circuit with nested lifetimes
fn short_circuit_nested<'r, 'w, T>(wrapper: &'r Wrapper<'w>, err_val: &'r str) -> MyResultNested<'r, 'w, T> {
    let _ = MyResultNested::Err(err_val)?;
    MyResultNested::Ok(wrapper)
}

// Lifetime hierarchy test: longer lifetime contains shorter ones
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultHierarchy<'long: 'short, 'short, T> {
    Ok(T),
    Err(&'short str),
}

// Test lifetime hierarchy (long lifetime contains short)
fn lifetime_hierarchy<'long: 'short, 'short, T>(ok_val: T, short_val: &'short str) -> MyResultHierarchy<'long, 'short, T> {
    let result = MyResultHierarchy::Ok(ok_val)?;
    MyResultHierarchy::Ok(result)
}

// Test short-circuit with shorter lifetime
fn short_circuit_shorter_lifetime<'long: 'short, 'short, T>(ok_val: T, short_val: &'short str) -> MyResultHierarchy<'long, 'short, T> {
    let _ = MyResultHierarchy::Err(short_val)?;
    MyResultHierarchy::Ok(ok_val)
}

// Higher-ranked trait bounds (HRTB) test
#[derive(Debug, Try, Try_ConvertResult)]
enum MyResultHRTB<T> {
    Ok(T),
    Err(&'static str),
}

// Test HRTB with closure
fn hrtb_closure<F>(f: F) -> MyResultHRTB<i32>
where
    F: for<'a> Fn(&'a str) -> MyResultHRTB<i32>,
{
    f("test")
}

// Test HRTB usage
fn test_hrtb() -> MyResultHRTB<i32> {
    hrtb_closure(|s| MyResultHRTB::Ok(42))
}

fn main() {
    let ok_val = 42;
    let err1_val = "error1";
    let err2_val = "error2";

    // Test complex cases
    let result1 = multiple_error_lifetimes(ok_val, &err1_val, &err2_val);
    let result2 = short_circuit_different_errors(ok_val, &err1_val, &err2_val);
    let result3 = short_circuit_other_error(ok_val, &err1_val, &err2_val);

    // Test nested lifetimes
    let wrapper = Wrapper { value: &ok_val };
    let result4 = nested_lifetimes(&wrapper, &err1_val);
    let result5 = short_circuit_nested(&wrapper, &err1_val);

    // Test lifetime hierarchy
    let result6 = lifetime_hierarchy(ok_val, &err1_val);
    let result7 = short_circuit_shorter_lifetime(ok_val, &err1_val);

    // Test HRTB
    let result8 = test_hrtb();

    // Verify results
    match result1 {
        MyResultComplex::Ok(val) => assert_eq!(val, 42),
        _ => panic!("Should not short-circuit"),
    }

    match result4 {
        MyResultNested::Ok(wrapper) => assert_eq!(*wrapper.value, 42),
        _ => panic!("Should not short-circuit"),
    }

    match result6 {
        MyResultHierarchy::Ok(val) => assert_eq!(val, 42),
        _ => panic!("Should not short-circuit"),
    }

    match result8 {
        MyResultHRTB::Ok(val) => assert_eq!(val, 42),
        _ => panic!("HRTB test failed"),
    }
}