use derive_more::{Deref, Constructor, DerefMut};
use serde::{Serialize, Deserialize};
use super::SignedEvent;

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Deref, DerefMut)]
pub struct SignedEvents(Vec<SignedEvent>);
