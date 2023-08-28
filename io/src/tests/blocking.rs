use super::common::*;
use crate::blocking::{prelude::*, Receiver, RecvError, Sender};
use flatty::vec::FromIterator;
use ringbuf_blocking::{traits::*, BlockingHeapRb};
use std::thread::spawn;

#[cfg(feature = "shared")]
use crate::blocking::shared::{SharedReceiver, SharedSender};
#[cfg(feature = "shared")]
use std::{mem::replace, thread::sleep};

fn pipe() -> BlockingHeapRb<u8> {
    BlockingHeapRb::new(17)
}

#[test]
fn unique() {
    let (prod, cons) = pipe().split();
    let (send, recv) = (
        spawn(move || {
            let mut sender = Sender::<TestMsg, _>::new(prod, MAX_SIZE);

            sender.alloc().default_in_place().unwrap().send().unwrap();

            sender.alloc().new_in_place(TestMsgInitB(123456)).unwrap().send().unwrap();

            sender
                .alloc()
                .new_in_place(TestMsgInitC(FromIterator(0..7)))
                .unwrap()
                .send()
                .unwrap();
        }),
        spawn(move || {
            let mut receiver = Receiver::<TestMsg, _>::new(cons, MAX_SIZE);

            match receiver.recv().unwrap().as_ref() {
                TestMsgRef::A => (),
                _ => panic!(),
            }

            match receiver.recv().unwrap().as_ref() {
                TestMsgRef::B(x) => assert_eq!(*x, 123456),
                _ => panic!(),
            }

            match receiver.recv().unwrap().as_ref() {
                TestMsgRef::C(v) => {
                    assert!(v.iter().copied().eq(0..7));
                }
                _ => panic!(),
            }

            match receiver.recv().err().unwrap() {
                RecvError::Eof => (),
                _ => panic!(),
            }
        }),
    );
    recv.join().unwrap();
    send.join().unwrap();
}

#[cfg(feature = "shared")]
#[test]
fn shared_sender() {
    let (prod, cons) = pipe().split();
    let mut sender = SharedSender::<TestMsg, _>::new(prod, MAX_SIZE);
    let mut receiver = Receiver::<TestMsg, _>::new(cons, MAX_SIZE);

    let (sends, recv) = (
        [
            spawn({
                let mut sender = sender.clone();
                move || {
                    sender
                        .alloc_message()
                        .new_in_place(TestMsgInitB(123456))
                        .unwrap()
                        .send()
                        .unwrap();
                }
            }),
            spawn(move || {
                sender
                    .alloc_message()
                    .new_in_place(TestMsgInitC(FromIterator(0..7)))
                    .unwrap()
                    .send()
                    .unwrap();
            }),
        ],
        spawn(move || {
            let mut prev = TestMsgTag::A;
            for _ in 0..2 {
                let message = receiver.recv().unwrap();
                let tag = message.tag();
                assert_ne!(replace(&mut prev, tag), tag);
                match message.as_ref() {
                    TestMsgRef::B(x) => {
                        assert_eq!(*x, 123456);
                    }
                    TestMsgRef::C(v) => {
                        assert!(v.iter().copied().eq(0..7));
                    }
                    _ => panic!(),
                }
            }

            match receiver.recv().err().unwrap() {
                RecvError::Eof => (),
                _ => panic!(),
            }
        }),
    );
    recv.join().unwrap();
    for send in sends {
        send.join().unwrap();
    }
}

#[cfg(feature = "shared")]
#[test]
fn shared_receiver() {
    const ATTEMPTS: usize = 16;

    let (prod, cons) = pipe().split();
    let mut sender = Sender::<TestMsg, _>::new(prod, MAX_SIZE);
    let receiver = SharedReceiver::<TestMsg, _>::new(cons, MAX_SIZE);

    let (send, recvs) = (
        spawn(move || {
            for i in 0..ATTEMPTS {
                sender.alloc().new_in_place(TestMsgInitB(i as i32)).unwrap().send().unwrap();

                sender
                    .alloc()
                    .new_in_place(TestMsgInitC(FromIterator((1..8).map(|x| x * (i + 1) as i32))))
                    .unwrap()
                    .send()
                    .unwrap();
            }
        }),
        [
            spawn({
                let mut receiver = receiver.clone().filter(|m| m.tag() == TestMsgTag::B);
                move || {
                    sleep(TIMEOUT);

                    for i in 0..ATTEMPTS {
                        match receiver.recv().unwrap().as_ref() {
                            TestMsgRef::B(x) => {
                                assert_eq!(*x, i as i32);
                            }
                            _ => panic!(),
                        }
                    }

                    match receiver.recv().err().unwrap() {
                        RecvError::Eof => (),
                        _ => panic!(),
                    }
                }
            }),
            spawn({
                let mut receiver = receiver.filter(|m| m.tag() == TestMsgTag::C);
                move || {
                    for i in 0..ATTEMPTS {
                        match receiver.recv().unwrap().as_ref() {
                            TestMsgRef::C(v) => {
                                assert!(v.iter().copied().eq((1..8).map(|x| x * (i + 1) as i32)));
                            }
                            _ => panic!(),
                        }
                    }

                    match receiver.recv().err().unwrap() {
                        RecvError::Eof => (),
                        _ => panic!(),
                    }
                }
            }),
        ],
    );

    for recv in recvs {
        recv.join().unwrap();
    }
    send.join().unwrap();
}
