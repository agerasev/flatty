pub type SendError<E> = E;

#[derive(Debug)]
pub enum RecvError<E> {
    Read(E),
    Parse(flatty::Error),
    Closed,
}
