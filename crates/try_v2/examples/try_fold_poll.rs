use std::task::Poll;

fn main() {
    let polls = [
        Poll::Ready(Ok(0)),
        Poll::Pending,
        Poll::Ready(Err(2)),
        Poll::Ready(Ok(3)),
    ];
    let _ = polls.into_iter().try_fold(
        Poll::Ready(0),
        |total: Poll<i32>, n: Poll<Result<i32, i32>>| -> Result<Poll<i32>, i32> {
            let n: Poll<i32> = n?; // <- shorts on Poll::Ready(Err)
            let total: Poll<i32> = match n {
                Poll::Ready(n) => total.map(|prev| prev + n),
                Poll::Pending => Poll::Pending,
            };
            Ok(total)
        },
    );
}
