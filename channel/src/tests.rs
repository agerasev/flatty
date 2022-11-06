use async_ringbuf::AsyncHeapRb;
use flatty::{
    flat,
    portable::{le, NativeCast},
    vec::FromIterator,
    FlatVec,
};
use futures::{executor::block_on, join};

use super::{reader::ReadError, writer::AbstractWriter, AsyncReader, AsyncWriter};

#[flat(sized = false, portable = true, default = true)]
enum TestMsg {
    #[default]
    A,
    B(le::I32),
    C(FlatVec<le::I32, le::U16>),
}

#[test]
fn channel() {
    block_on(async {
        const MAX_SIZE: usize = 32;
        let (prod, cons) = AsyncHeapRb::<u8>::new(17).split();
        join!(
            async move {
                let mut writer = AsyncWriter::<TestMsg, _>::new(prod, MAX_SIZE);

                writer.new_message().default().unwrap().write_async().await.unwrap();

                {
                    writer
                        .new_message()
                        .emplace(TestMsgInitB(le::I32::from(123456)))
                        .unwrap()
                        .write_async()
                        .await
                        .unwrap();
                }

                {
                    writer
                        .new_message()
                        .emplace(TestMsgInitC(FromIterator((0..7).into_iter().map(le::I32::from))))
                        .unwrap()
                        .write_async()
                        .await
                        .unwrap();
                }
            },
            async move {
                let mut reader = AsyncReader::<TestMsg, _>::new(cons, MAX_SIZE);

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
                        TestMsgRef::B(x) => assert_eq!(x.to_native(), 123456),
                        _ => panic!(),
                    }
                }

                {
                    let guard = reader.read_message().await.unwrap();
                    match guard.as_ref() {
                        TestMsgRef::C(v) => {
                            assert!(v.iter().map(|x| x.to_native()).eq((0..7).into_iter()));
                        }
                        _ => panic!(),
                    }
                }

                match reader.read_message().await.err().unwrap() {
                    ReadError::Eof => (),
                    _ => panic!(),
                }
            },
        );
    });
}
