use super::common::*;
use crate::async_::{Receiver, RecvError, Sender};
use async_ringbuf::{traits::*, AsyncHeapRb};
use async_std::{task::spawn, test as async_test};
use flatty::vec::FromIterator;
use futures::join;
/*
#[cfg(feature = "shared")]
use crate::async_::shared::{SharedReceiver, SharedSender};
#[cfg(feature = "shared")]
use async_std::task::sleep;
#[cfg(feature = "shared")]
use std::mem::replace;
*/
fn pipe() -> AsyncHeapRb<u8> {
    AsyncHeapRb::<u8>::new(17)
}

#[async_test]
async fn unique() {
    let (prod, cons) = pipe().split();
    join!(
        spawn(async move {
            let mut sender = Sender::<TestMsg, _>::io(prod, MAX_SIZE);

            sender
                .alloc_()
                .await
                .unwrap()
                .default_in_place()
                .unwrap()
                .send_()
                .await
                .unwrap();

            sender
                .alloc_()
                .await
                .unwrap()
                .new_in_place(TestMsgInitB(123456))
                .unwrap()
                .send_()
                .await
                .unwrap();

            sender
                .alloc_()
                .await
                .unwrap()
                .new_in_place(TestMsgInitC(FromIterator(0..7)))
                .unwrap()
                .send_()
                .await
                .unwrap();
        }),
        spawn(async move {
            let mut receiver = Receiver::<TestMsg, _>::io(cons, MAX_SIZE);

            {
                let guard = receiver.recv_().await.unwrap();
                match guard.as_ref() {
                    TestMsgRef::A => (),
                    _ => panic!(),
                }
            }

            {
                let guard = receiver.recv_().await.unwrap();
                match guard.as_ref() {
                    TestMsgRef::B(x) => assert_eq!(*x, 123456),
                    _ => panic!(),
                }
            }

            {
                let guard = receiver.recv_().await.unwrap();
                match guard.as_ref() {
                    TestMsgRef::C(v) => {
                        assert!(v.iter().copied().eq(0..7));
                    }
                    _ => panic!(),
                }
            }

            match receiver.recv_().await.err().unwrap() {
                RecvError::Closed => (),
                _ => panic!(),
            }
        })
    );
}
/*
#[cfg(feature = "shared")]
#[async_test]
async fn shared_sender() {
    let (prod, cons) = pipe().split();
    let mut sender = SharedSender::<TestMsg, _>::new(prod, MAX_SIZE);
    let mut receiver = Receiver::<TestMsg, _>::new(cons, MAX_SIZE);

    join!(
        spawn({
            let mut sender = sender.clone();
            async move {
                sender
                    .alloc()
                    .new_in_place(TestMsgInitB(123456))
                    .unwrap()
                    .send()
                    .await
                    .unwrap();
            }
        }),
        spawn(async move {
            sender
                .alloc()
                .new_in_place(TestMsgInitC(FromIterator(0..7)))
                .unwrap()
                .send()
                .await
                .unwrap();
        }),
        spawn(async move {
            let mut prev = TestMsgTag::A;
            for _ in 0..2 {
                let message = receiver.recv().await.unwrap();
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

            match receiver.recv().await.err().unwrap() {
                RecvError::Eof => (),
                _ => panic!(),
            }
        }),
    );
}

#[cfg(feature = "shared")]
#[async_test]
async fn shared_receiver() {
    const ATTEMPTS: usize = 16;

    let (prod, cons) = pipe().split();
    let mut sender = Sender::<TestMsg, _>::new(prod, MAX_SIZE);
    let receiver = SharedReceiver::<TestMsg, _>::new(cons, MAX_SIZE);

    join!(
        spawn(async move {
            for i in 0..ATTEMPTS {
                sender
                    .alloc()
                    .new_in_place(TestMsgInitB(i as i32))
                    .unwrap()
                    .send()
                    .await
                    .unwrap();

                sender
                    .alloc()
                    .new_in_place(TestMsgInitC(FromIterator((1..8).map(|x| x * (i + 1) as i32))))
                    .unwrap()
                    .send()
                    .await
                    .unwrap();
            }
        }),
        spawn({
            let mut receiver = receiver.clone().filter(|m| m.tag() == TestMsgTag::B);
            async move {
                sleep(TIMEOUT).await;

                for i in 0..ATTEMPTS {
                    match receiver.recv().await.unwrap().as_ref() {
                        TestMsgRef::B(x) => {
                            assert_eq!(*x, i as i32);
                        }
                        _ => panic!(),
                    }
                }

                match receiver.recv().await.err().unwrap() {
                    RecvError::Eof => (),
                    _ => panic!(),
                }
            }
        }),
        spawn({
            let mut receiver = receiver.filter(|m| m.tag() == TestMsgTag::C);
            async move {
                for i in 0..ATTEMPTS {
                    match receiver.recv().await.unwrap().as_ref() {
                        TestMsgRef::C(v) => {
                            assert!(v.iter().copied().eq((1..8).map(|x| x * (i + 1) as i32)));
                        }
                        _ => panic!(),
                    }
                }

                match receiver.recv().await.err().unwrap() {
                    RecvError::Eof => (),
                    _ => panic!(),
                }
            }
        }),
    );
}
*/
