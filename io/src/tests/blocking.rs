use super::common::*;
use crate::{prelude::*, BlockingSharedReader, BlockingSharedWriter, ReadError, Reader, Writer};
use flatty::vec::FromIterator;
use ringbuf_blocking::{traits::*, BlockingHeapRb};
use std::{
    mem::replace,
    thread::{sleep, spawn},
};

fn pipe() -> BlockingHeapRb<u8> {
    BlockingHeapRb::new(17)
}

#[test]
fn unique() {
    let (prod, cons) = pipe().split();
    let (write, read) = (
        spawn(move || {
            let mut writer = Writer::<TestMsg, _>::new(prod, MAX_SIZE);

            writer.alloc_message().default_in_place().unwrap().write().unwrap();

            writer
                .alloc_message()
                .new_in_place(TestMsgInitB(123456))
                .unwrap()
                .write()
                .unwrap();

            writer
                .alloc_message()
                .new_in_place(TestMsgInitC(FromIterator(0..7)))
                .unwrap()
                .write()
                .unwrap();
        }),
        spawn(move || {
            let mut reader = Reader::<TestMsg, _>::new(cons, MAX_SIZE);

            match reader.read_message().unwrap().as_ref() {
                TestMsgRef::A => (),
                _ => panic!(),
            }

            match reader.read_message().unwrap().as_ref() {
                TestMsgRef::B(x) => assert_eq!(*x, 123456),
                _ => panic!(),
            }

            match reader.read_message().unwrap().as_ref() {
                TestMsgRef::C(v) => {
                    assert!(v.iter().copied().eq(0..7));
                }
                _ => panic!(),
            }

            match reader.read_message().err().unwrap() {
                ReadError::Eof => (),
                _ => panic!(),
            }
        }),
    );
    read.join().unwrap();
    write.join().unwrap();
}

#[test]
fn shared_writer() {
    let (prod, cons) = pipe().split();
    let mut writer = BlockingSharedWriter::<TestMsg, _>::new(prod, MAX_SIZE);
    let mut reader = Reader::<TestMsg, _>::new(cons, MAX_SIZE);

    let (writes, read) = (
        [
            spawn({
                let mut writer = writer.clone();
                move || {
                    writer
                        .alloc_message()
                        .new_in_place(TestMsgInitB(123456))
                        .unwrap()
                        .write()
                        .unwrap();
                }
            }),
            spawn(move || {
                writer
                    .alloc_message()
                    .new_in_place(TestMsgInitC(FromIterator(0..7)))
                    .unwrap()
                    .write()
                    .unwrap();
            }),
        ],
        spawn(move || {
            let mut prev = TestMsgTag::A;
            for _ in 0..2 {
                let message = reader.read_message().unwrap();
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

            match reader.read_message().err().unwrap() {
                ReadError::Eof => (),
                _ => panic!(),
            }
        }),
    );
    read.join().unwrap();
    for write in writes {
        write.join().unwrap();
    }
}

#[test]
fn shared_reader() {
    const ATTEMPTS: usize = 16;

    let (prod, cons) = pipe().split();
    let mut writer = Writer::<TestMsg, _>::new(prod, MAX_SIZE);
    let reader = BlockingSharedReader::<TestMsg, _>::new(cons, MAX_SIZE);

    let (write, reads) = (
        spawn(move || {
            for i in 0..ATTEMPTS {
                writer
                    .alloc_message()
                    .new_in_place(TestMsgInitB(i as i32))
                    .unwrap()
                    .write()
                    .unwrap();

                writer
                    .alloc_message()
                    .new_in_place(TestMsgInitC(FromIterator((1..8).map(|x| x * (i + 1) as i32))))
                    .unwrap()
                    .write()
                    .unwrap();
            }
        }),
        [
            spawn({
                let mut reader = reader.clone().filter(|m| m.tag() == TestMsgTag::B);
                move || {
                    sleep(TIMEOUT);

                    for i in 0..ATTEMPTS {
                        match reader.read_message().unwrap().as_ref() {
                            TestMsgRef::B(x) => {
                                assert_eq!(*x, i as i32);
                            }
                            _ => panic!(),
                        }
                    }

                    match reader.read_message().err().unwrap() {
                        ReadError::Eof => (),
                        _ => panic!(),
                    }
                }
            }),
            spawn({
                let mut reader = reader.filter(|m| m.tag() == TestMsgTag::C);
                move || {
                    for i in 0..ATTEMPTS {
                        match reader.read_message().unwrap().as_ref() {
                            TestMsgRef::C(v) => {
                                assert!(v.iter().copied().eq((1..8).map(|x| x * (i + 1) as i32)));
                            }
                            _ => panic!(),
                        }
                    }

                    match reader.read_message().err().unwrap() {
                        ReadError::Eof => (),
                        _ => panic!(),
                    }
                }
            }),
        ],
    );

    for read in reads {
        read.join().unwrap();
    }
    write.join().unwrap();
}
