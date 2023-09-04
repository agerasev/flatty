use core::ops::DerefMut;

pub trait WriteBuffer: DerefMut<Target = [u8]> {
    type Error;
}

pub type SendError<E> = E;
