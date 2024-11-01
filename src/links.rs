#![allow(unused)]

use std::sync::{Arc, Mutex};

pub type LinkData = Box<[u8]>;

type Locked<T> = Arc<Mutex<T>>;
#[derive(PartialEq, Eq, Debug)]
pub enum LinkError {
    ReceiverAlreadyAttached,
    LinkIsDown,
}

pub struct LinkEnd {
    link: Locked<dyn Link>,
    end_id: LinkEndId,
}

impl LinkEnd {
    fn new(end_id: LinkEndId, link: Locked<dyn Link>) -> Self {
        Self { link, end_id }
    }
    fn get_link_id(&self) -> LinkEndId {
        self.end_id
    }
    pub fn send(&self, data: &[u8]) -> Result<(), LinkError> {
        let mut link = self.link.lock().unwrap();
        link.send(self.end_id, data)
    }
    pub fn attach_receiver<F>(&self, handler: F) -> Result<(), LinkError>
    where
        F: FnMut(LinkData) + Send + Sync + 'static, // TODO: make it so the lifetime of this as to
                                                    // be as long as this function lives
    {
        let mut link = self.link.lock().unwrap();
        link.attach_receiver(self.end_id, Box::new(handler))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum LinkEndId {
    First = 0,
    Second = 1,
}

impl LinkEndId {
    #[inline]
    pub fn get_other_end(self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }
}

type LinkEndHandler = Box<dyn FnMut(LinkData) + Send + Sync>;
pub trait Link {
    fn send(&mut self, from: LinkEndId, data: &[u8]) -> Result<(), LinkError>;
    fn attach_receiver(
        &mut self,
        link_end: LinkEndId,
        handler: LinkEndHandler,
    ) -> Result<(), LinkError>;
}

pub fn create_link() -> (LinkEnd, LinkEnd) {
    let link = Arc::new(Mutex::new(SimpleLink::new()));
    (
        LinkEnd::new(LinkEndId::First, link.clone()),
        LinkEnd::new(LinkEndId::Second, link),
    )
}

pub struct SimpleLink {
    receivers: [Option<LinkEndHandler>; 2],
}

impl SimpleLink {
    fn new() -> Self {
        Self {
            receivers: [None, None],
        }
    }
}

impl Link for SimpleLink {
    fn send(&mut self, from: LinkEndId, data: &[u8]) -> Result<(), LinkError> {
        let to = from.get_other_end();
        let idx = to as usize;

        match &mut self.receivers[idx] {
            Some(handler) => {
                handler(Box::from(data));
                Ok(())
            }
            None => Err(LinkError::LinkIsDown),
        }
    }

    fn attach_receiver(
        &mut self,
        link_end: LinkEndId,
        handler: LinkEndHandler,
    ) -> Result<(), LinkError> {
        let idx = link_end as usize;
        match self.receivers[idx] {
            Some(_) => Err(LinkError::ReceiverAlreadyAttached),
            None => {
                self.receivers[idx] = Some(handler);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{LinkEnd, LinkError};
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    fn attach_receiver(link_end: &LinkEnd) -> Arc<AtomicU32> {
        let value = Arc::new(AtomicU32::new(0));
        let copy = Arc::clone(&value);
        link_end
            .attach_receiver(move |data| {
                copy.store(
                    u32::from_ne_bytes(
                        data.as_ref()
                            .try_into()
                            .expect("Failed to parse bytes as u32"),
                    ),
                    Ordering::Relaxed,
                )
            })
            .expect("Failed to attach receiver");
        value
    }

    #[test]
    fn creation_and_attachment() {
        let (end_1, end_2) = super::create_link();

        assert_ne!(end_1.get_link_id(), end_2.get_link_id());

        assert_eq!(Ok(()), end_1.attach_receiver(|_| {}));
        assert_eq!(Ok(()), end_2.attach_receiver(|_| {}));

        assert_eq!(
            Err(LinkError::ReceiverAlreadyAttached),
            end_1.attach_receiver(|_| {})
        );
        assert_eq!(
            Err(LinkError::ReceiverAlreadyAttached),
            end_2.attach_receiver(|_| {})
        );
    }

    #[test]
    fn send_and_receive() {
        let (end_1, end_2) = super::create_link();

        assert_eq!(Err(LinkError::LinkIsDown), end_1.send(&0u32.to_ne_bytes()));
        assert_eq!(Err(LinkError::LinkIsDown), end_2.send(&0u32.to_ne_bytes()));

        let v1 = attach_receiver(&end_1);
        let v2 = attach_receiver(&end_2);

        assert_eq!(v1.load(Ordering::Relaxed), 0);
        assert_eq!(v2.load(Ordering::Relaxed), 0);

        assert_eq!(Ok(()), end_1.send(&10u32.to_ne_bytes()));

        assert_eq!(v1.load(Ordering::Relaxed), 0);
        assert_eq!(v2.load(Ordering::Relaxed), 10);

        assert_eq!(Ok(()), end_2.send(&11u32.to_ne_bytes()));

        assert_eq!(v1.load(Ordering::Relaxed), 11);
        assert_eq!(v2.load(Ordering::Relaxed), 10);
    }
}
