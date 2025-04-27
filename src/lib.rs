#![no_std]
use core::{pin::Pin, task::Poll};

use futures::Stream;
use generic_array::ArrayLength;
// pin_project_lite::pin_project! {
pub struct Concurrent<S: Stream, const N: usize> {
    // #[pin]
    fetch: S,
    // #[pin]
    bank: [Option<S::Item>; N],
    // idx: usize,
}
impl<S: Stream<Item: Stream>, const N: usize> Stream for Concurrent<S, N> {
    type Item = <S::Item as Stream>::Item;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        loop {
            'a: for b2 in unsafe { self.as_mut().get_unchecked_mut() }.bank.iter_mut() {
                if let Some(b) = b2.as_mut() {
                    let b = unsafe { Pin::new_unchecked(b) };
                    let Poll::Ready(r) = b.poll_next(cx) else {
                        break 'a;
                    };
                    match r {
                        None => {
                            *b2 = None;
                        }
                        Some(v) => {
                            return Poll::Ready(Some(v));
                        }
                    }
                }
            }
            if self.bank.iter().all(|a| a.is_some()) {
                return Poll::Pending;
            }
            let Poll::Ready(a) =
                unsafe { Pin::new_unchecked(&mut self.as_mut().get_unchecked_mut().fetch) }
                    .poll_next(cx)
            else {
                continue;
            };
            let Some(a) = a else {
                if self.bank.iter().all(|a| a.is_none()) {
                    return Poll::Ready(None);
                } else {
                    continue;
                }
            };
            for b in unsafe { self.as_mut().get_unchecked_mut() }.bank.iter_mut() {
                if let None = b {
                    *b = Some(a);
                    break;
                }
            }
        }
    }
}
// };

pub struct GConcurrent<S: Stream, N: ArrayLength> {
    // #[pin]
    fetch: S,
    // #[pin]
    bank: generic_array::GenericArray<Option<S::Item>, N>,
    // idx: usize,
}
impl<S: Stream<Item: Stream>, N: ArrayLength> Stream for GConcurrent<S, N> {
    type Item = <S::Item as Stream>::Item;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        loop {
            'a: for b2 in unsafe { self.as_mut().get_unchecked_mut() }.bank.iter_mut() {
                if let Some(b) = b2.as_mut() {
                    let b = unsafe { Pin::new_unchecked(b) };
                    let Poll::Ready(r) = b.poll_next(cx) else {
                        break 'a;
                    };
                    match r {
                        None => {
                            *b2 = None;
                        }
                        Some(v) => {
                            return Poll::Ready(Some(v));
                        }
                    }
                }
            }
            if self.bank.iter().all(|a| a.is_some()) {
                return Poll::Pending;
            }
            let Poll::Ready(a) =
                unsafe { Pin::new_unchecked(&mut self.as_mut().get_unchecked_mut().fetch) }
                    .poll_next(cx)
            else {
                continue;
            };
            let Some(a) = a else {
                if self.bank.iter().all(|a| a.is_none()) {
                    return Poll::Ready(None);
                } else {
                    continue;
                }
            };
            for b in unsafe { self.as_mut().get_unchecked_mut() }.bank.iter_mut() {
                if let None = b {
                    *b = Some(a);
                    break;
                }
            }
        }
    }
}
// };
pub trait StreamExt: Stream {
    fn concurrent<const N: usize>(self) -> Concurrent<Self, N>
    where
        Self: Sized,
    {
        Concurrent {
            fetch: self,
            bank: [const { None }; N],
        }
    }
    fn typenum_concurrent<N: ArrayLength>(self) -> GConcurrent<Self, N>
    where
        Self: Sized,
    {
        GConcurrent {
            fetch: self,
            bank: generic_array::GenericArray::from_iter(core::iter::from_fn(|| Some(None))),
        }
    }
}
impl<T: Stream + ?Sized> StreamExt for T {}
