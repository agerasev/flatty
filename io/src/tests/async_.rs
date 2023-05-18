use super::common::*;
use crate::{AsyncReader, AsyncWriter, ReadError};
use async_ringbuf::AsyncHeapRb;
use flatty::vec::FromIterator;
use futures::{executor::block_on, join};

#[test]
fn test() {
    block_on(async {
        const MAX_SIZE: usize = 36;
        let (prod, cons) = AsyncHeapRb::<u8>::new(17).split();
        join!(
            async move {
                let mut writer = AsyncWriter::<TestMsg, _>::new(prod, MAX_SIZE);

                writer.alloc_message().default().unwrap().write().await.unwrap();

                {
                    writer
                        .alloc_message()
                        .new_in_place(TestMsgInitB(123456))
                        .unwrap()
                        .write()
                        .await
                        .unwrap();
                }

                {
                    writer
                        .alloc_message()
                        .new_in_place(TestMsgInitC(FromIterator(0..7)))
                        .unwrap()
                        .write()
                        .await
                        .unwrap();
                }
            },
            async move {
                let mut reader = AsyncReader::<TestMsg, _>::new(cons, MAX_SIZE);

                {
                    let guard = reader.read_message().await.unwrap();
                    match guard.as_ref() {
                        TestMsgRef::A => (),
                        _ => panic!(),
                    }
                }

                {
                    let guard = reader.read_message().await.unwrap();
                    match guard.as_ref() {
                        TestMsgRef::B(x) => assert_eq!(*x, 123456),
                        _ => panic!(),
                    }
                }

                {
                    let guard = reader.read_message().await.unwrap();
                    match guard.as_ref() {
                        TestMsgRef::C(v) => {
                            assert!(v.iter().copied().eq(0..7));
                        }
                        _ => panic!(),
                    }
                }

                match reader.read_message().await.err().unwrap() {
                    ReadError::Eof => (),
                    _ => panic!(),
                }
            },
        );
    });
}
