use super::common::*;
use crate::{ReadError, Reader, Writer};
use flatty::{
    portable::{le, NativeCast},
    vec::FromIterator,
};
use pipe::pipe;
use std::thread;

#[test]
fn test() {
    const MAX_SIZE: usize = 32;
    let (cons, prod) = pipe();
    let (write, read) = (
        thread::spawn(move || {
            let mut writer = Writer::<TestMsg, _>::new(prod, MAX_SIZE);

            writer.new_message().default().unwrap().write().unwrap();

            {
                writer
                    .new_message()
                    .emplace(TestMsgInitB(le::I32::from(123456)))
                    .unwrap()
                    .write()
                    .unwrap();
            }

            {
                writer
                    .new_message()
                    .emplace(TestMsgInitC(FromIterator((0..7).map(le::I32::from))))
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
                    TestMsgRef::B(x) => assert_eq!(x.to_native(), 123456),
                    _ => panic!(),
                }
            }

            {
                let guard = reader.read_message().unwrap();
                match guard.as_ref() {
                    TestMsgRef::C(v) => {
                        assert!(v.iter().map(|x| x.to_native()).eq(0..7));
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
