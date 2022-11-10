use std::{
    env::args,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use anyhow::{Context, Error, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpSocket, TcpStream,
    },
    spawn,
};

#[tokio::main]
async fn main() -> Result<()> {
    let bind = args().nth(1).context("Arg 1 should be bind addr")?;
    let conn = args().nth(2).context("Arg 2 should be cind addr")?;
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
    let (arx, atx) = a.into_split();
    let (brx, btx) = b.into_split();
    spawn(pipe_half(arx, btx));
    spawn(pipe_half(brx, atx));
    Ok(())
}

async fn pipe_half(mut rx: OwnedReadHalf, mut tx: OwnedWriteHalf) {
    let mut buf = [0; 1024];
    while let Ok(size) = rx.read(&mut buf).await {
        if size == 0 {
            break;
        } else {
            if let Err(e) = tx.write_all(&buf[..size]).await {
                println!("{e}");
            };
        }
    }
}
