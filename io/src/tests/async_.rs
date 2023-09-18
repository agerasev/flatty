use super::common::*;
use crate::{AsyncReceiver, AsyncSender, RecvError};
use async_ringbuf::{traits::*, AsyncHeapRb};
use async_std::{task::spawn, test as async_test};
use flatty::vec::FromIterator;
use futures::join;

#[async_test]
async fn unique() {
    let (prod, cons) = AsyncHeapRb::<u8>::new(17).split();
    join!(
        spawn(async move {
            let mut sender = AsyncSender::<TestMsg, _>::io(prod, MAX_SIZE);

            sender
                .alloc()
                .await
                .unwrap()
                .default_in_place()
                .unwrap()
                .send()
                .await
                .unwrap();

            sender
                .alloc()
                .await
                .unwrap()
                .new_in_place(TestMsgInitB(123456))
                .unwrap()
                .send()
                .await
                .unwrap();

            sender
                .alloc()
                .await
                .unwrap()
                .new_in_place(TestMsgInitC(FromIterator(0..7)))
                .unwrap()
                .send()
                .await
                .unwrap();
        }),
        spawn(async move {
            let mut receiver = AsyncReceiver::<TestMsg, _>::io(cons, MAX_SIZE);

            {
                let guard = receiver.recv().await.unwrap();
                match guard.as_ref() {
                    TestMsgRef::A => (),
                    _ => panic!(),
                }
            }

            {
                let guard = receiver.recv().await.unwrap();
                match guard.as_ref() {
                    TestMsgRef::B(x) => assert_eq!(*x, 123456),
                    _ => panic!(),
                }
            }

            {
                let guard = receiver.recv().await.unwrap();
                match guard.as_ref() {
                    TestMsgRef::C(v) => {
                        assert!(v.iter().copied().eq(0..7));
                    }
                    _ => panic!(),
                }
            }

            match receiver.recv().await.err().unwrap() {
                RecvError::Closed => (),
                _ => panic!(),
            }
        })
    );
}
