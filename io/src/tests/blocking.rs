use super::common::*;
use crate::{prelude::*, ReadError};
use flatty::vec::FromIterator;
use pipe::pipe;
use std::thread;

#[cfg(feature = "test_shared")]
use crate::{BlockingSharedReader as Reader, BlockingSharedWriter as Writer};
#[cfg(not(feature = "test_shared"))]
use crate::{Reader, Writer};

#[test]
fn test() {
    const MAX_SIZE: usize = 36;
    let (cons, prod) = pipe();
    let (write, read) = (
        thread::spawn(move || {
            let mut writer = Writer::<TestMsg, _>::new(prod, MAX_SIZE);

            writer.alloc_message().default_in_place().unwrap().write().unwrap();

            {
                writer
                    .alloc_message()
                    .new_in_place(TestMsgInitB(123456))
                    .unwrap()
                    .write()
                    .unwrap();
            }

            {
                writer
                    .alloc_message()
                    .new_in_place(TestMsgInitC(FromIterator(0..7)))
                    .unwrap()
                    .write()
                    .unwrap();
            }
        }),
        thread::spawn(move || {
            let mut reader = Reader::<TestMsg, _>::new(cons, MAX_SIZE);

            {
                let guard = reader.read_message().unwrap();
                match guard.as_ref() {
                    TestMsgRef::A => (),
                    _ => panic!(),
                }
            }

            {
                let guard = reader.read_message().unwrap();
                match guard.as_ref() {
                    TestMsgRef::B(x) => assert_eq!(*x, 123456),
                    _ => panic!(),
                }
            }

            {
                let guard = reader.read_message().unwrap();
                match guard.as_ref() {
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
    write.join().unwrap();
}
