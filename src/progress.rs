use iced::futures::{channel::mpsc, StreamExt};
use iced::{subscription, Subscription};

use crate::{command, Message};

pub enum ProgressState {
    Starting,
    Ready(mpsc::UnboundedReceiver<String>),
}

pub fn bind() -> Subscription<Message> {
    struct Progress;

    subscription::unfold(
        std::any::TypeId::of::<Progress>(),
        ProgressState::Starting,
        |state| async move {
            match state {
                ProgressState::Starting => {
                    let (sender, receiver) = mpsc::unbounded();

                    (Message::Ready(sender), ProgressState::Ready(receiver))
                }
                ProgressState::Ready(mut progress_receiver) => {
                    let received = progress_receiver.next().await;
                    if let Some(progress) = received {
                        if progress.contains("Finished") {
                            return (
                                Message::Command(command::Message::Finished),
                                ProgressState::Ready(progress_receiver),
                            );
                        }

                        return (
                            Message::ProgressEvent(progress),
                            ProgressState::Ready(progress_receiver),
                        );
                    }

                    (Message::None, ProgressState::Ready(progress_receiver))
                }
            }
        },
    )
}
