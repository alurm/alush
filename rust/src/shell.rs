// use std::iter::Peekable;
// use std::sync::mpsc::{self, SyncSender, Sender, Receiver};
// use std::thread;

// use crate::grammar::{self};

// // pub fn shell(i: &mut Input, o: Sender<grammar::Statement>) -> Option<()> {
// //     loop {
// //         let s = grammar::statement(i);
// //         o.send(s?);
// //     }
// // }

// // #[test]
// // fn test_shell() {
// //     let (send_u8, recv_u8) = mpsc::channel::<u8>();
// //     // let (send_stmt, recv_stmt) = mpsc::channel();

// //     thread::spawn(move || {
// //         let i = recv_u8.into_iter();
// //         let b = Box::new(i) as Box<dyn Iterator<Item = u8>>;
// //         let mut p = b.peekable();
// //         // shell(&mut p, send_stmt);
// //     });

// //     // dbg!(recv_stmt.recv().unwrap());
// //     // dbg!(recv_stmt.recv().unwrap());
// //     // dbg!(recv_stmt.recv().unwrap());

// //     // h.join();
// // }

// // #[test]
// // fn mpsc() -> Result<(), ()> {
// //     let (send, recv) = mpsc::sync_channel::<u8>(10);
// //     send.send(10);
// //     dbg!(recv.iter().peekable().peek());
// //     // recv.recv().unwrap();
// //     Ok(())
// // }

// // pub fn statements(input: &mut Peekable<impl Iterator<Item = u8>>) -> Option<Statements> {
// //     let result = Vec::<u8>::new();
// // }

// // #[test]
// // fn peekable() {
// //     let s = String::from("a b c");
// //     let mut peekable = s.bytes().into_iter().peekable();
// //     dbg!(*peekable.peek().unwrap() as char);
// //     dbg!(*peekable.peek().unwrap() as char);
// // }
