use tokio::io::{BufReader, ReadHalf};

pub type ReadHalfBuf<T> = BufReader<ReadHalf<T>>;
