use super::common::*;
use crate::{prelude::*, BlockingSharedReader, BlockingSharedWriter, ReadError};
use flatty::vec::FromIterator;
use pipe::pipe;
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

#[test]
fn unique() {
    let (cons, prod) = pipe();
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
fn shared_writer() {
    let (cons, prod) = pipe();
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
    let (cons, prod) = pipe();
    let mut writer = Writer::<TestMsg, _>::new(prod, MAX_SIZE);
    let reader = BlockingSharedReader::<TestMsg, _>::new(cons, MAX_SIZE);

    let (write, reads) = (
        thread::spawn(move || {
            writer
                .alloc_message()
                .new_in_place(TestMsgInitB(123456))
                .unwrap()
                .write()
                .unwrap();
            println!("Write B");

            writer
                .alloc_message()
                .new_in_place(TestMsgInitC(FromIterator(0..7)))
                .unwrap()
                .write()
                .unwrap();
            println!("Write C");
        }),
        [
            thread::spawn({
                let mut reader = reader.clone().filter(|m| m.tag() == TestMsgTag::B);
                move || {
                    println!("Start B");
                    sleep(TIMEOUT);

                    match reader.read_message().unwrap().as_ref() {
                        TestMsgRef::B(x) => {
                            println!("Read B");
                            assert_eq!(*x, 123456);
                        }
                        _ => panic!(),
                    }

                    match reader.read_message().err().unwrap() {
                        ReadError::Eof => (),
                        _ => panic!(),
                    }
                    println!("Eof B");
                }
            }),
            thread::spawn({
                let mut reader = reader.filter(|m| m.tag() == TestMsgTag::C);
                move || {
                    println!("Start C");
                    match reader.read_message().unwrap().as_ref() {
                        TestMsgRef::C(v) => {
                            println!("Read C");
                            assert!(v.iter().copied().eq(0..7));
                        }
                        _ => panic!(),
                    }

                    match reader.read_message().err().unwrap() {
                        ReadError::Eof => (),
                        _ => panic!(),
                    }
                    println!("Eof C");
                }
            }),
        ],
    );
    println!("Spawned");
    for read in reads {
        read.join().unwrap();
    }
    write.join().unwrap();
}
