// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Imports
//======================================================================================================================

use crate::{collections::async_value::SharedAsyncValue, runtime::fail::Fail};
use ::socket2::Type;
use ::std::net::SocketAddr;

//======================================================================================================================
// Structures
//======================================================================================================================

#[derive(Copy, Clone, Debug, PartialEq)]
enum SocketState {
    Unbound,
    Bound(SocketAddr),
    /// Bound to a local address and is able to accept incoming connections.
    PassiveListening {
        local: SocketAddr,
    },
    /// Connecting to a remote address.
    ActiveConnecting {
        local: Option<SocketAddr>,
        remote: SocketAddr,
    },
    /// Connected to a remote address.
    ActiveEstablished {
        local: Option<SocketAddr>,
        remote: SocketAddr,
    },
    Closing,
    Closed,
}

#[derive(Clone, Debug)]
pub struct SocketStateMachine {
    typ: Type,
    current: SharedAsyncValue<SocketState>,
}

//======================================================================================================================
// Associated Functions
//======================================================================================================================

impl SocketStateMachine {
    pub fn new_unbound(typ: Type) -> Self {
        debug_assert!(typ == Type::STREAM || typ == Type::DGRAM);
        Self {
            typ,
            current: SharedAsyncValue::new(SocketState::Unbound),
        }
    }

    pub fn new_established(remote: SocketAddr) -> Self {
        Self {
            typ: Type::STREAM,
            current: SharedAsyncValue::new(SocketState::ActiveEstablished { local: None, remote }),
        }
    }

    pub fn may_bind(&self) -> Result<(), Fail> {
        match self.current.get() {
            SocketState::Unbound => Ok(()),
            SocketState::Closing | SocketState::Closed => Err(fail("socket is closing", libc::EBADF)),
            _ => Err(fail("socket is already bound", libc::EINVAL)),
        }
    }

    pub fn bind(&mut self, local: SocketAddr) {
        self.current.set(SocketState::Bound(local))
    }

    pub fn may_listen(&self) -> Result<(), Fail> {
        match self.current.get() {
            SocketState::Unbound => Err(fail("socket is not bound", libc::EDESTADDRREQ)),
            SocketState::Bound(_) => Ok(()),
            SocketState::PassiveListening { local: _ } => Err(fail("socket is already listening", libc::EADDRINUSE)),
            SocketState::ActiveConnecting { local: _, remote: _ } => {
                Err(fail("socket is already connecting", libc::EADDRINUSE))
            },
            SocketState::ActiveEstablished { local: _, remote: _ } => {
                Err(fail("socket is already connected", libc::EISCONN))
            },
            _ => Err(fail("socket is closing", libc::EBADF)),
        }
    }

    pub fn listen(&mut self) {
        let new_state: SocketState = match self.current.get() {
            SocketState::Bound(local) => SocketState::PassiveListening { local },
            _ => return,
        };
        self.current.set(new_state)
    }

    pub fn may_accept(&self) -> Result<(), Fail> {
        self.ensure_not_closing()?;
        self.ensure_not_closed()?;
        self.ensure_listening()?;
        Ok(())
    }

    pub async fn while_may_accept(&mut self) -> Fail {
        loop {
            match self.may_accept() {
                Ok(()) => {
                    // Check state again if it changes.
                    _ = self.current.clone().wait_for_change(None).await;
                    continue;
                },
                Err(e) => return e,
            }
        }
    }

    pub fn may_connect(&self) -> Result<(), Fail> {
        match self.current.get() {
            SocketState::Unbound | SocketState::Bound(_) => Ok(()),
            SocketState::ActiveConnecting { local: _, remote: _ } => {
                Err(fail("socket already is already connecting", libc::EINPROGRESS))
            },
            SocketState::PassiveListening { local: _ } => Err(fail("socket is already listening", libc::EOPNOTSUPP)),
            SocketState::ActiveEstablished { local: _, remote: _ } => {
                Err(fail("socket is already connected", libc::EISCONN))
            },
            _ => Err(fail("socket is closing", libc::EBADF)),
        }
    }

    pub fn connecting(&mut self, remote: SocketAddr) {
        let local: Option<SocketAddr> = match self.current.get() {
            SocketState::Bound(local) => Some(local),
            _ => None,
        };
        self.current.set(SocketState::ActiveConnecting { local, remote })
    }

    pub fn connected(&mut self, remote: SocketAddr) {
        let local: Option<SocketAddr> = match self.current.get() {
            SocketState::Bound(local) => Some(local),
            SocketState::ActiveConnecting { local, remote: _ } => local,
            _ => None,
        };

        self.current.set(SocketState::ActiveEstablished { local, remote });
    }

    pub async fn while_open(&mut self) -> Fail {
        while self.ensure_not_closing().is_ok() && self.ensure_not_closed().is_ok() {
            // Check state again if it changes.
            _ = self.current.clone().wait_for_change(None).await;
        }
        Fail::new(libc::EBADF, "socket is closing")
    }

    pub async fn while_closing(&mut self) -> Fail {
        while self.ensure_not_closed().is_ok() {
            // Check state again if it changes.
            _ = self.current.clone().wait_for_change(None).await;
        }
        Fail::new(libc::EBADF, "socket is closed")
    }

    pub fn may_push(&self) -> Result<(), Fail> {
        self.ensure_not_closing()?;
        self.ensure_not_closed()?;

        if self.typ == Type::STREAM {
            self.ensure_established()?;
        }

        // NOTE: no need to ensure other states, because this is checked on the prepare operation.

        Ok(())
    }

    pub fn may_pop(&self) -> Result<(), Fail> {
        self.ensure_not_closing()?;
        self.ensure_not_closed()?;

        if self.typ == Type::STREAM {
            self.ensure_established()?;
        } else {
            self.ensure_bound()?;
        }

        // NOTE: no need to ensure other states, because this is checked on the prepare operation.

        Ok(())
    }

    pub fn closing(&mut self) {
        self.current.set(SocketState::Closing)
    }

    pub fn closed(&mut self) {
        self.current.set(SocketState::Closed)
    }

    fn ensure_bound(&self) -> Result<(), Fail> {
        match self.current.get() {
            SocketState::Bound(_) => Ok(()),
            _ => Err(fail("socket is not bound", libc::EDESTADDRREQ)),
        }
    }

    fn ensure_listening(&self) -> Result<(), Fail> {
        match self.current.get() {
            SocketState::PassiveListening { local: _ } => Ok(()),
            _ => Err(fail("socket is not listening", libc::EINVAL)),
        }
    }

    fn ensure_established(&self) -> Result<(), Fail> {
        match self.current.get() {
            SocketState::ActiveEstablished { local: _, remote: _ } => Ok(()),
            _ => Err(fail("socket is not connected", libc::ENOTCONN)),
        }
    }

    pub fn ensure_not_closing(&self) -> Result<(), Fail> {
        match self.current.get() {
            SocketState::Closing => Err(fail("socket is closing", libc::EBADF)),
            _ => Ok(()),
        }
    }

    fn ensure_not_closed(&self) -> Result<(), Fail> {
        match self.current.get() {
            SocketState::Closed => Err(fail("socket is closed", libc::EBADF)),
            _ => Ok(()),
        }
    }

    fn get_state(&self) -> SocketState {
        self.current.get()
    }

    pub fn local(&self) -> Option<SocketAddr> {
        match self.current.get() {
            SocketState::Bound(local) | SocketState::PassiveListening { local } => Some(local),
            SocketState::ActiveEstablished { local, remote: _ }
            | SocketState::ActiveConnecting { local, remote: _ } => local,
            _ => None,
        }
    }

    pub fn remote(&self) -> Option<SocketAddr> {
        match self.current.get() {
            SocketState::ActiveConnecting { local: _, remote }
            | SocketState::ActiveEstablished { local: _, remote } => Some(remote),
            _ => None,
        }
    }
}

//======================================================================================================================
// Trait Implementations
//======================================================================================================================

impl PartialEq for SocketStateMachine {
    fn eq(&self, other: &Self) -> bool {
        self.get_state() == other.get_state()
    }
}

//======================================================================================================================
// Standalone Functions
//======================================================================================================================

fn fail(cause: &str, errno: i32) -> Fail {
    error!("{}", cause);
    Fail::new(errno, cause)
}
