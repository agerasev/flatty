use super::common::*;
use crate::blocking::{Receiver, RecvError, Sender};
use flatty::vec::FromIterator;
use ringbuf_blocking::{traits::*, BlockingHeapRb};
use std::thread::spawn;

#[test]
fn unique() {
    let (prod, cons) = BlockingHeapRb::<u8>::new(17).split();
    let (send, recv) = (
        spawn(move || {
            let mut sender = Sender::<TestMsg, _>::io(prod, MAX_SIZE);

            sender.alloc().unwrap().default_in_place().unwrap().send().unwrap();

            sender
                .alloc()
                .unwrap()
                .new_in_place(TestMsgInitB(123456))
                .unwrap()
                .send()
                .unwrap();

            sender
                .alloc()
                .unwrap()
                .new_in_place(TestMsgInitC(FromIterator(0..7)))
                .unwrap()
                .send()
                .unwrap();
        }),
        spawn(move || {
            let mut receiver = Receiver::<TestMsg, _>::io(cons, MAX_SIZE);

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
                RecvError::Closed => (),
                other => panic!("{:?}", other),
            }
        }),
    );
    recv.join().unwrap();
    send.join().unwrap();
}
