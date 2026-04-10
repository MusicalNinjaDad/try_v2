#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::Try;

#[derive(Try)]
#[must_use]
enum Eightball<Y> {
    Yes(Box<Y>),
    No,
}

fn ring() {
    enum IsBoxed<T> {
        Yes(Box<T>),
        No,
    }
    let bar: IsBoxed<!> = IsBoxed::No;

    let _: IsBoxed<!> = match bar {
        IsBoxed::No => IsBoxed::No,
        IsBoxed::Yes(erm) => *erm,
    };
    
    enum IsVec<T> {
        Yes(Vec<T>),
        No,
    }

    let foo: IsVec<!> = IsVec::No;

    let _: IsVec<!> = match foo {
        IsVec::No => IsVec::No,
        IsVec::Yes(_) => unreachable!("contained !")
    };
}