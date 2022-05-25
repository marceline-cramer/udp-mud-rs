use clap::Parser;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use protocol::*;
use protocol_derive::{Decode, Encode};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

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
pub struct Pronouns {
    /// Non-zero for true, zero for false.
    pub case_sensitive: u8, // TODO bitfield or bool protocol impl?

    /// Ex. he, she, they, fae.
    pub subject: String,

    /// Ex. him, her, them, faer.
    pub object: String,

    /// Ex. his, her, their, faer.
    pub possessive: String,

    /// Ex. his, hers, theirs, faers.
    pub possessive_pronoun: String,

    /// Ex. himself, herself, themself, faerself.
    pub reflexive: String,
}

#[derive(Debug, Decode, Encode)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub about: String,
    pub pronouns: Pronouns,
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
    owned_rooms: HashMap<String, Room>,
    remote_rooms: HashMap<String, Room>,
}

impl App {
    pub fn new(args: Args) -> Self {
        let socket = UdpSocket::bind(args.bind_addr).unwrap();
        let mut app = Self {
            args,
            owned_rooms: Default::default(),
            remote_rooms: Default::default(),
            socket,
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
            }
        };

        self.owned_rooms.insert(room.info.id.clone(), room);
    }

    pub fn run(mut self) {
        let mut buf = [0u8; 65507];
        while let Ok((len, from)) = self.socket.recv_from(&mut buf) {
            let mut buf = buf.as_slice();
            println!("recv'd from {:?}: {:?}", from, &buf[..len]);

            let kind = Var::<u16>::decode(&mut buf).unwrap().0;
            let kind: PacketKind = match kind.try_into() {
                Ok(kind) => kind,
                Err(int) => {
                    eprintln!("unrecognized packet kind {}", int);
                    continue;
                }
            };

            self.on_packet(from, kind, buf);
        }
    }

    pub fn on_packet(&mut self, from: SocketAddr, kind: PacketKind, mut reader: &[u8]) {
        println!("handling {:?}", kind);
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
            kind => eprintln!("unimplemented packet handler for {:?}", kind),
        }
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
