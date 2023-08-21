use super::common::*;
use crate::{prelude::*, BlockingSharedReader, BlockingSharedWriter, ReadError};
use flatty::vec::FromIterator;
use ringbuf_blocking::{traits::*, BlockingHeapRb};
use std::{
    mem::replace,
    thread::{self, sleep},
    time::Duration,
};

#[cfg(feature = "test_shared")]
use crate::{BlockingSharedReader as Reader, BlockingSharedWriter as Writer};
#[cfg(not(feature = "test_shared"))]
use crate::{Reader, Writer};

const MAX_SIZE: usize = 36;

const TIMEOUT: Duration = Duration::from_millis(10);

fn pipe() -> BlockingHeapRb<u8> {
    BlockingHeapRb::new(17)
}

#[test]
fn unique() {
    let (prod, cons) = pipe().split();
    let (write, read) = (
        thread::spawn(move || {
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
        thread::spawn(move || {
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
fn condvar() {
    use std::sync::{Arc, Condvar, Mutex};

    const ATTEMPTS: usize = 256;

    let mutex = Arc::new(Mutex::<usize>::new(0));
    let cvs = (Arc::new(Condvar::new()), Arc::new(Condvar::new()));

    let threads = [
        thread::spawn({
            let mutex = mutex.clone();
            let cvs = (cvs.1.clone(), cvs.0.clone());
            move || {
                let mut guard = mutex.lock().unwrap();
                for i in 0..ATTEMPTS {
                    guard = cvs.0.wait(guard).unwrap();
                    assert_eq!(*guard, 2 * i);
                    *guard += 1;
                    cvs.1.notify_one();
                }
            }
        }),
        thread::spawn({
            move || {
                sleep(TIMEOUT);
                cvs.1.notify_one();

                let mut guard = mutex.lock().unwrap();
                for i in 0..ATTEMPTS {
                    guard = cvs.0.wait(guard).unwrap();
                    assert_eq!(*guard, 2 * i + 1);
                    *guard += 1;
                    cvs.1.notify_one();
                }
            }
        }),
    ];
    for t in threads {
        t.join().unwrap();
    }
}

#[test]
fn shared_writer() {
    let (prod, cons) = pipe().split();
    let mut writer = BlockingSharedWriter::<TestMsg, _>::new(prod, MAX_SIZE);
    let mut reader = Reader::<TestMsg, _>::new(cons, MAX_SIZE);

    let (writes, read) = (
        [
            thread::spawn({
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
            thread::spawn(move || {
                writer
                    .alloc_message()
                    .new_in_place(TestMsgInitC(FromIterator(0..7)))
                    .unwrap()
                    .write()
                    .unwrap();
            }),
        ],
        thread::spawn(move || {
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
        thread::spawn(move || {
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
            thread::spawn({
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
            thread::spawn({
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
