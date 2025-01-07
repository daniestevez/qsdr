use anyhow::Result;
use futures::stream::{FusedStream, Stream};
use pin_project_lite::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

macro_rules! generate {
    ($Sequence:ident, <$($S:ident),*>, $sequence:ident, ($($stream:ident),*)) => {
        pin_project! {
            pub struct $Sequence<$($S),*> {
                $(
                    #[pin]
                    $stream: $S
                ),*,
                done: bool,
            }
        }

        #[allow(clippy::too_many_arguments)]
        pub fn $sequence<$($S: FusedStream),*>($($stream: $S),*) -> $Sequence<$($S),*> {
            let done = (|| {
                $(
                    if !$stream.is_terminated() {
                        return false;
                    }
                )*
                true
            })();
            $Sequence { $($stream),*, done }
        }

        impl<E, $($S),*> Stream for $Sequence<$($S),*>
        where $($S: Stream<Item=Result<(), E>> + FusedStream),*
        {
            type Item = Result<(), E>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut all_done = true;
                // Only return pending when all the streams return
                // pending. Otherwise one of the streams might be able to be
                // polled immediately again.
                let mut all_pending = true;
                let self_ = self.project();
                $(
                    if !self_.$stream.is_terminated() {
                        match self_.$stream.poll_next(cx) {
                            Poll::Ready(Some(Ok(()))) => {
                                all_done = false;
                                all_pending = false;
                            }
                            Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                            Poll::Ready(None) => all_pending = false,
                            Poll::Pending => {}
                        }
                    }
                )*
                if all_pending {
                    return Poll::Pending;
                }
                if all_done {
                    *self_.done = true;
                    return Poll::Ready(None);
                }
                Poll::Ready(Some(Ok(())))
            }
        }

        impl<E, $($S),*> FusedStream for $Sequence<$($S),*>
        where $($S: Stream<Item=Result<(), E>> + FusedStream),*
        {
            fn is_terminated(&self) -> bool {
                return self.done
            }
        }
    }
}

generate!(Sequence2, <S1, S2>,
          sequence2, (stream1, stream2));
generate!(Sequence3, <S1, S2, S3>,
          sequence3, (stream1, stream2, stream3));
generate!(Sequence4, <S1, S2, S3, S4>,
          sequence4, (stream1, stream2, stream3, stream4));
generate!(Sequence5, <S1, S2, S3, S4, S5>,
          sequence5, (stream1, stream2, stream3, stream4, stream5));
generate!(Sequence6, <S1, S2, S3, S4, S5, S6>,
          sequence6, (stream1, stream2, stream3, stream4, stream5, stream6));
generate!(Sequence7, <S1, S2, S3, S4, S5, S6, S7>,
          sequence7, (stream1, stream2, stream3, stream4, stream5, stream6, stream7));
generate!(Sequence8, <S1, S2, S3, S4, S5, S6, S7, S8>,
          sequence8, (stream1, stream2, stream3, stream4, stream5, stream6, stream7, stream8));
