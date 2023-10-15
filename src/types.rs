use tokio::{
    io::{BufReader, ReadHalf},
    net::TcpStream,
};

pub type ReadHalfBuf = BufReader<ReadHalf<TcpStream>>;
