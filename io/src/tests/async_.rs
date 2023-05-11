use super::common::*;
use crate::{AsyncReader, AsyncWriter, ReadError};
use async_ringbuf::AsyncHeapRb;
use flatty::{
    portable::{le, NativeCast},
    vec::FromIterator,
};
use futures::{executor::block_on, join};

#[test]
fn test() {
    block_on(async {
        const MAX_SIZE: usize = 32;
        let (prod, cons) = AsyncHeapRb::<u8>::new(17).split();
        join!(
            async move {
                let mut writer = AsyncWriter::<TestMsg, _>::new(prod, MAX_SIZE);

                writer.new_message().default().unwrap().write().await.unwrap();

                {
                    writer
                        .new_message()
                        .emplace(TestMsgInitB(le::I32::from(123456)))
                        .unwrap()
                        .write()
                        .await
                        .unwrap();
                }

                {
                    writer
                        .new_message()
                        .emplace(TestMsgInitC(FromIterator((0..7).map(le::I32::from))))
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
                        TestMsgRef::B(x) => assert_eq!(x.to_native(), 123456),
                        _ => panic!(),
                    }
                }

                {
                    let guard = reader.read_message().await.unwrap();
                    match guard.as_ref() {
                        TestMsgRef::C(v) => {
                            assert!(v.iter().map(|x| x.to_native()).eq(0..7));
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
