use std::{sync::Arc, time::Duration};

use tokio::net::UdpSocket;

mod sigserv;
mod stun;


#[tokio::main]
async fn main() {
    println!("Получение адреса от STUN сервера...");
    let stun_addr = stun::send_binding_request("0.0.0.0:0", "stun2.l.google.com:19302")
        .await
        .expect("Не удалось получить адрес от STUN сервера");
    println!("{:#?}", stun_addr);
    let soket_addr = format!("0.0.0.0:{}", stun_addr.port);
    let soket = Arc::new(UdpSocket::bind(soket_addr)
        .await
        .unwrap());
    let resiv_soket = soket.clone();
    let soket_jh = tokio::spawn(async move {
        loop {
            let mut buf = [0; 1024];
            match resiv_soket.recv(&mut buf).await {
                Ok(len) => {
                    let received_data = buf[..len].to_vec();
                    if let Ok(mess) = String::from_utf8(received_data) {
                        println!("{}", mess)
                    }
                },
                Err(_) => break
            }
        }
    });
    let input_jh = tokio::spawn(async move {
        let input = loop {
            println!("[1] Создать канал\n[2] Присоединиться\nВвод:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Ошибка ввода");
            match input.trim().parse::<u8>() {
                Ok(v) if (1..=2).contains(&v) => break v,
                _ => println!("\nОшибка ввода, попробуйте снова"),
            }
        };
        let chan_id = match input {
            1 => {
                let chan_id = sigserv::create_chan(&stun_addr.ip, &stun_addr.port)
                .await
                .expect("\nНе удалось создать канал");
                println!("\nКанал успешно создан\nID: {}", chan_id);
                chan_id
            }
            2 => {
                println!("\nВведите ID канала:");
                let mut chan_id = String::new();
                std::io::stdin().read_line(&mut chan_id)
                    .expect("\nОшибка ввода");
                chan_id = chan_id.trim().into();
                let chan = sigserv::chan_join(&chan_id, &stun_addr.ip, &stun_addr.port)
                    .await
                    .expect("\nНе удалось присоединиться к каналу");
                println!("{:#?}", chan);
                if chan.peers.len() <= 1 {
                    println!("\nВведен несуществующий канал\nСоздан новый канал");
                }
                chan_id
            }
            _ => todo!()
        };
        println!("\nИщем пир для подключения в канале");
        let peer = loop {
            let chan = sigserv::get_chan(&chan_id).await.expect("\nНе удалось создать канал");
            if chan.peers.len() > 1 {
                break chan.peers.into_iter()
                    .filter(|p| p.port != stun_addr.port)
                    .collect::<Vec<_>>()[0]
                    .clone();
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        };
        println!("\nПопытка соединиться с пиром\n{:#?}", peer);
        let peer_bind_addr = peer.to_string();//format!("127.0.0.1:{}", peer.port);
        soket.connect(&peer_bind_addr).await.expect("Не удалось подключиться к пиру");
        for _ in 0..4 {
            soket.send(b"ping")
                .await
                .expect("Не удалось отпрвить сообщение пиру");
        }
        println!("\nСоединение  установлено\nОтправка сообщений пиру:");
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("stdin error");
            soket.send(input.as_bytes()).await.expect("Не удалось отпрвить сообщение пиру");
        }
    });

    let _ = soket_jh.await;
    let _ = input_jh.await;
}
