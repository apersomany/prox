use std::{
    env::args,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use anyhow::{Context, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpSocket, TcpStream},
    spawn,
};

#[tokio::main]
async fn main() -> Result<()> {
    let bind = args().nth(1).context("Arg 1 should be bind addr")?;
    let conn = args().nth(1).context("Arg 2 should be cind addr")?;
    let bind = SocketAddr::from_str(&bind)?;
    let conn = SocketAddr::from_str(&conn)?;

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, bind.port())).await?;
    while let Ok((strm, _)) = listener.accept().await {
        let sock = TcpSocket::new_v4()?;
        sock.bind((bind.ip(), 0).into())?;
        pipe(strm, sock.connect(conn).await?)?;
    }
    Ok(())
}

fn pipe(a: TcpStream, b: TcpStream) -> Result<()> {
    let (mut arx, mut atx) = a.into_split();
    let (mut brx, mut btx) = b.into_split();
    spawn(async move {
        let mut buf = [0; 1024];
        loop {
            let size = arx.read(&mut buf).await.unwrap();
            btx.write_all(&buf[..size]).await.unwrap();
        }
    });
    spawn(async move {
        let mut buf = [0; 1024];
        loop {
            let size = brx.read(&mut buf).await.unwrap();
            atx.write_all(&buf[..size]).await.unwrap();
        }
    });
    Ok(())
}
