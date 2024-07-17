use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use termion::raw::IntoRawMode;

pub fn listener() {
    // 標準出力をrawモードに切り替え
    let _stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdin = io::stdin().lock();
    let fd = stdin.as_raw_fd();

    // mioのPollを作成
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    // stdinのファイルディスクリプタを登録
    poll.registry()
        .register(
            &mut SourceFd(&fd),
            Token(0),
            Interest::READABLE | Interest::WRITABLE,
        )
        .unwrap();

    println!("Reading bytes from stdin (press Ctrl-C to exit):");

    let mut buffer = [0; 1];

    loop {
        // イベントを待機
        poll.poll(&mut events, None).unwrap();

        for event in &events {
            if event.token() == Token(0) && event.is_readable() {
                // stdinからバイトを読み取る
                match stdin.read(&mut buffer) {
                    Ok(1) => {
                        // 1バイト読み取ったら表示
                        print!("{:?}", buffer[0]);
                        io::stdout().flush().unwrap();
                    }
                    Ok(_) => {
                        // 0バイト読み取った場合（EOFなど）
                    }
                    Err(e) => {
                        // エラー処理
                        eprintln!("Error reading stdin: {}", e);
                        break;
                    }
                }
            }
        }
    }
}
