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
    pipe_half(arx, btx);
    pipe_half(brx, atx);
    Ok(())
}

fn pipe_half(mut srx: OwnedReadHalf, mut stx: OwnedWriteHalf) {
    let (ctx, crx) = flume::unbounded();
    spawn(async move {
        loop {
            let mut buf = Vec::with_capacity(1024);
            if srx.read_buf(&mut buf).await? == 0 {
                break;
            } else {
                ctx.send_async(buf).await?;
            }
        }
        Ok::<(), Error>(())
    });
    spawn(async move {
        loop {
            stx.write_all(&crx.recv_async().await?).await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), Error>(())
    });
}
