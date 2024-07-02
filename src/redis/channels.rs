use tokio::sync::{broadcast, mpsc, oneshot};

use crate::redis::cmd::Command;

type Receiver = mpsc::Receiver<Message>;

type Propagator = broadcast::Sender<Command>;
type ReplListener = broadcast::Receiver<Command>;

type GiveMeReplListener = oneshot::Sender<ReplListener>;

#[derive(Debug)]
pub(crate) enum Message {
    Handshake(GiveMeReplListener),
    Propagate(Command),
}

pub(crate) struct ChannelManager {
    receiver: Receiver,
    propagator: Propagator,
}

impl ChannelManager {
    pub(crate) fn new(receiver: Receiver) -> ChannelManager {
        let (tx, _) = broadcast::channel(16);
        ChannelManager { receiver, propagator: tx }
    }

    pub(crate) async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
           match msg {
               Message::Handshake(sender) => self.handle_handshake(sender),
               Message::Propagate(msg) => self.handle_propagate(msg).await,
           };
       }
    }
    async fn handle_propagate(&mut self, cmd: Command) {
        // propagate write command to all subscribers
        dbg!(&cmd);
        self.propagator.send(cmd).unwrap();
    }

    fn handle_handshake(&mut self, sender: GiveMeReplListener) {
        // on handshake -> respond with replication listener
        sender.send(self.propagator.subscribe()).unwrap();
    }
}