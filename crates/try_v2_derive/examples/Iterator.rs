#![allow(unused)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(iterator_try_collect)]

use std::ops::Try;

use try_v2_derive::Try;

#[derive(Debug, Try)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

impl<Y, N> IntoIterator for EightBall<Y, N> {
    type Item = <EightBall<Y, N> as std::ops::Try>::Output;
    type IntoIter = ::std::option::IntoIter<<EightBall<Y, N> as std::ops::Try>::Output>;

    fn into_iter(self) -> Self::IntoIter {
        match self.branch() {
            std::ops::ControlFlow::Continue(v) => Some(v),
            std::ops::ControlFlow::Break(_) => None,
        }
        .into_iter()
    }
}

impl<V, Y, N> FromIterator<EightBall<Y, N>> for EightBall<V, N>
where
    V: FromIterator<<EightBall<Y, N> as std::ops::Try>::Output>,
{
    fn from_iter<T: IntoIterator<Item = EightBall<Y, N>>>(iter: T) -> Self {
        iter.into_iter().try_collect()
    }
}

#[test]
fn into() {
    assert_eq!(Some(5), EightBall::<i32,i32>::Yes(5).into_iter().next());
    assert_eq!(None, EightBall::<i32,i32>::RollAgain.into_iter().next());
    assert_eq!(None, EightBall::<i32,i32>::No(5).into_iter().next());
}

fn main() {}
