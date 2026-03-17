use try_v2::Try;

#[derive(Try)]
struct NotAnEnum;

#[derive(Try)]
union AlsoNotAnEnum {
    foo: u8,
    bar: u8,
}
