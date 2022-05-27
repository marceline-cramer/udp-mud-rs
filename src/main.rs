use clap::Parser;
use crossbeam_channel::{Receiver, Sender};
use cursive::Cursive;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use protocol::*;
use protocol_derive::{Decode, Encode};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

mod pronouns;
mod tui;

#[derive(Parser, Debug)]
#[clap(
    author = "Marceline Cramer",
    about = "Experimental distributed UDP chat."
)]
pub struct Args {
    /// Name you appear as to other peers.
    #[clap(short, long)]
    pub username: String,

    /// Address to bind to.
    #[clap(short, long)]
    pub bind_addr: SocketAddr,

    /// Other address to initiate connection with.
    #[clap(short, long)]
    pub connect: Option<SocketAddr>,
}

#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum PacketKind {
    Ping,
    Pong,
    RequestUserInfo,
    RequestRoomInfo,
    RequestRoomList,
    UserInfo,
    RoomInfo,
    RoomList,
    Message,
}

#[derive(Debug, Decode, Encode)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub about: String,
}

#[derive(Debug, Decode, Encode)]
pub struct RoomInfo {
    pub id: String,
    pub title: String,
    pub short_about: String,
    pub long_about: String,
}

#[derive(Debug, Decode, Encode)]
pub struct RoomList {
    pub room_ids: Vec<String>,
}

#[derive(Debug, Decode, Encode)]
pub struct Message {
    pub sender: String,
    pub contents: String,
}

pub struct Room {
    pub info: RoomInfo,
}

pub struct App {
    args: Args,
    socket: UdpSocket,
    cursive: Cursive,
    owned_rooms: HashMap<String, Room>,
    remote_rooms: HashMap<String, Room>,
    message_sender: Sender<String>,
    message_receiver: Receiver<String>,
    // TODO connection management
    other: Option<SocketAddr>,
}

impl App {
    pub fn new(args: Args) -> Self {
        let socket = UdpSocket::bind(args.bind_addr).unwrap();
        socket.set_nonblocking(true).unwrap();

        let (message_sender, message_receiver) = crossbeam_channel::unbounded();

        let cursive = tui::make_cursive(message_sender.to_owned());

        let mut app = Self {
            args,
            socket,
            cursive,
            owned_rooms: Default::default(),
            remote_rooms: Default::default(),
            message_sender,
            message_receiver,
            other: None,
        };
        app.startup();
        app
    }

    pub fn startup(&mut self) {
        if let Some(connect) = self.args.connect.as_ref() {
            self.send_empty_packet(connect, PacketKind::Ping).unwrap();
        }

        let room = Room {
            info: RoomInfo {
                id: format!("{}_owned_room", self.args.username),
                title: format!("{}'s Bombass Owned Room", self.args.username),
                short_about: "An automatically-created room for testing.".into(),
                long_about: "".into(),
            },
        };

        self.owned_rooms.insert(room.info.id.clone(), room);
    }

    pub fn run(mut self) {
        let mut siv = tui::make_cursive(self.message_sender.to_owned());
        let siv_backend = cursive::backends::try_default().unwrap();
        let mut siv_runner = siv.runner(siv_backend);
        siv_runner.refresh();

        while siv_runner.is_running() {
            siv_runner.step();

            let mut buf = [0u8; 65507];
            // TODO error handling of non-non-blocking errors
            if let Ok((len, from)) = self.socket.recv_from(&mut buf) {
                let mut buf = buf.as_slice();

                let kind = Var::<u16>::decode(&mut buf).unwrap().0;
                let kind: PacketKind = match kind.try_into() {
                    Ok(kind) => kind,
                    Err(int) => {
                        eprintln!("unrecognized packet kind {}", int);
                        continue;
                    }
                };

                if let Some(message) = self.on_packet(from, kind, buf) {
                    tui::add_message(&mut siv_runner, &message);
                    siv_runner.refresh(); // TODO better refresh management
                }
            }

            if let Some(other) = self.other.as_ref() {
                while let Ok(message) = self.message_receiver.try_recv() {
                    eprintln!("sending message: {}", message);
                    self.send_packet(other, PacketKind::Message, |writer| {
                        let message = Message {
                            sender: self.args.username.clone(),
                            contents: message,
                        };
                        message.encode(writer)
                    })
                    .unwrap();
                }
            }
        }
    }

    pub fn on_packet(&mut self, from: SocketAddr, kind: PacketKind, mut reader: &[u8]) -> Option<Message> {
        println!("handling {:?}", kind);

        // TODO proper connection management
        self.other = Some(from);

        match kind {
            PacketKind::Ping => self.send_empty_packet(from, PacketKind::Pong).unwrap(),
            PacketKind::Pong => self
                .send_empty_packet(from, PacketKind::RequestRoomList)
                .unwrap(),
            PacketKind::RequestRoomList => self
                .send_packet(from, PacketKind::RoomList, |writer| {
                    let room_list = self.build_room_list();
                    room_list.encode(writer)
                })
                .unwrap(),
            PacketKind::RoomList => {
                let room_list: RoomList = Decode::decode(&mut reader).unwrap();
                for room_id in room_list.room_ids.iter() {
                    self.send_packet(from, PacketKind::RequestRoomInfo, |writer| {
                        room_id.encode(writer)
                    })
                    .unwrap();
                }
            }
            PacketKind::RequestRoomInfo => {
                let room_id = String::decode(&mut reader).unwrap();
                if let Some(room) = self.owned_rooms.get(&room_id) {
                    self.send_packet(from, PacketKind::RoomInfo, |writer| {
                        room.info.encode(writer)
                    })
                    .unwrap();
                } else {
                    eprintln!("Unrecognized room info request for {}", room_id);
                }
            }
            PacketKind::RoomInfo => {
                let info = RoomInfo::decode(&mut reader).unwrap();
                eprintln!("Received room info: {:#?}", info);
                self.remote_rooms.insert(info.id.clone(), Room { info });
            }
            PacketKind::Message => {
                let message = Message::decode(&mut reader).unwrap();
                return Some(message);
            }
            kind => eprintln!("unimplemented packet handler for {:?}", kind),
        }

        None
    }

    pub fn build_room_list(&self) -> RoomList {
        let room_ids: Vec<_> = self
            .owned_rooms
            .iter()
            .map(|(id, _room)| id.to_owned())
            .collect();
        RoomList { room_ids }
    }

    pub fn send_packet(
        &self,
        addr: impl ToSocketAddrs,
        kind: PacketKind,
        encode: impl FnOnce(&mut Vec<u8>) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        let mut buf = Vec::new();
        Var(kind as u16).encode(&mut buf)?;
        encode(&mut buf)?;
        self.socket.send_to(&buf, addr)?;
        Ok(())
    }

    pub fn send_empty_packet(
        &self,
        addr: impl ToSocketAddrs,
        kind: PacketKind,
    ) -> std::io::Result<()> {
        self.send_packet(addr, kind, |_| Ok(()))
    }
}

fn main() {
    let args = Args::parse();
    let app = App::new(args);
    app.run();
}
