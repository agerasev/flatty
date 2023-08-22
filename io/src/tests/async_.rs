use std::mem::replace;

use super::common::*;
use crate::{prelude::*, reader::AsyncSharedReader, AsyncSharedWriter, ReadError, Reader, Writer};
use async_ringbuf::{traits::*, AsyncHeapRb};
use async_std::{
    task::{sleep, spawn},
    test as async_test,
};
use flatty::vec::FromIterator;
use futures::join;

fn pipe() -> AsyncHeapRb<u8> {
    AsyncHeapRb::<u8>::new(17)
}

#[async_test]
async fn unique() {
    let (prod, cons) = pipe().split();
    join!(
        spawn(async move {
            let mut writer = Writer::<TestMsg, _>::new(prod, MAX_SIZE);

            writer.alloc_message().default_in_place().unwrap().write().await.unwrap();

            writer
                .alloc_message()
                .new_in_place(TestMsgInitB(123456))
                .unwrap()
                .write()
                .await
                .unwrap();

            writer
                .alloc_message()
                .new_in_place(TestMsgInitC(FromIterator(0..7)))
                .unwrap()
                .write()
                .await
                .unwrap();
        }),
        spawn(async move {
            let mut reader = Reader::<TestMsg, _>::new(cons, MAX_SIZE);

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
        })
    );
}

#[async_test]
async fn shared_writer() {
    let (prod, cons) = pipe().split();
    let mut writer = AsyncSharedWriter::<TestMsg, _>::new(prod, MAX_SIZE);
    let mut reader = Reader::<TestMsg, _>::new(cons, MAX_SIZE);

    join!(
        spawn({
            let mut writer = writer.clone();
            async move {
                writer
                    .alloc_message()
                    .new_in_place(TestMsgInitB(123456))
                    .unwrap()
                    .write()
                    .await
                    .unwrap();
            }
        }),
        spawn(async move {
            writer
                .alloc_message()
                .new_in_place(TestMsgInitC(FromIterator(0..7)))
                .unwrap()
                .write()
                .await
                .unwrap();
        }),
        spawn(async move {
            let mut prev = TestMsgTag::A;
            for _ in 0..2 {
                let message = reader.read_message().await.unwrap();
                let tag = message.tag();
                assert_ne!(replace(&mut prev, tag), tag);
                match message.as_ref() {
                    TestMsgRef::B(x) => {
                        assert_eq!(*x, 123456);
                    }
                    TestMsgRef::C(v) => {
                        println!("@ {:?}", v);
                        assert!(v.iter().copied().eq(0..7));
                    }
                    _ => panic!(),
                }
            }

            match reader.read_message().await.err().unwrap() {
                ReadError::Eof => (),
                _ => panic!(),
            }
        }),
    );
}

#[async_test]
async fn shared_reader() {
    const ATTEMPTS: usize = 16;

    let (prod, cons) = pipe().split();
    let mut writer = Writer::<TestMsg, _>::new(prod, MAX_SIZE);
    let reader = AsyncSharedReader::<TestMsg, _>::new(cons, MAX_SIZE);

    join!(
        spawn(async move {
            for i in 0..ATTEMPTS {
                writer
                    .alloc_message()
                    .new_in_place(TestMsgInitB(i as i32))
                    .unwrap()
                    .write()
                    .await
                    .unwrap();

                writer
                    .alloc_message()
                    .new_in_place(TestMsgInitC(FromIterator((1..8).map(|x| x * (i + 1) as i32))))
                    .unwrap()
                    .write()
                    .await
                    .unwrap();
            }
        }),
        spawn({
            let mut reader = reader.clone().filter(|m| m.tag() == TestMsgTag::B);
            async move {
                sleep(TIMEOUT).await;

                for i in 0..ATTEMPTS {
                    match reader.read_message().await.unwrap().as_ref() {
                        TestMsgRef::B(x) => {
                            assert_eq!(*x, i as i32);
                        }
                        _ => panic!(),
                    }
                }

                match reader.read_message().await.err().unwrap() {
                    ReadError::Eof => (),
                    _ => panic!(),
                }
            }
        }),
        spawn({
            let mut reader = reader.filter(|m| m.tag() == TestMsgTag::C);
            async move {
                for i in 0..ATTEMPTS {
                    match reader.read_message().await.unwrap().as_ref() {
                        TestMsgRef::C(v) => {
                            assert!(v.iter().copied().eq((1..8).map(|x| x * (i + 1) as i32)));
                        }
                        _ => panic!(),
                    }
                }

                match reader.read_message().await.err().unwrap() {
                    ReadError::Eof => (),
                    _ => panic!(),
                }
            }
        }),
    );
}
