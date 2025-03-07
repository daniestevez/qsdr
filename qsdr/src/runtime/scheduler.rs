use anyhow::Result;
use futures::{TryStream, TryStreamExt};

mod sequence;
pub use sequence::{
    Sequence2, Sequence3, Sequence4, Sequence5, Sequence6, Sequence7, Sequence8, sequence2,
    sequence3, sequence4, sequence5, sequence6, sequence7, sequence8,
};

pub async fn run<S: TryStream<Ok = ()>>(stream: S) -> Result<(), <S as TryStream>::Error> {
    stream.try_collect::<()>().await
}
