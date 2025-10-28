use std::os::fd::{RawFd, AsRawFd};
use libc::{kqueue, kevent, kevent64_s, EV_ADD, EV_DELETE, EV_ENABLE, EVFILT_READ};
use crate::server::Server;

/// Create a new kqueue descriptor
pub fn create_kqueue() -> RawFd {
    let kq = unsafe { kqueue() };
    if kq == -1 {
        panic!("Failed to create kqueue: {}", std::io::Error::last_os_error());
    }
    kq
}

/// Register an event with the kqueue
pub fn register_event(kqueue: RawFd, fd: RawFd, filter: i16, flags: u16) -> std::io::Result<()> {
    let mut event = kevent64_s {
        ident: fd as u64,
        filter,
        flags,
        fflags: 0,
        data: 0,
        udata: 0,
        ext: [0, 0],
    };

    let result = unsafe {
        kevent(
            kqueue,
            &mut event as *mut _ as *const _,
            1,
            std::ptr::null_mut(),
            0,
            std::ptr::null(),
        )
    };

    if result == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Register all listening sockets for read events
pub fn register_listeners(server: &Server, kqueue: RawFd) {
    for socket in &server.sockets {
        let fd = socket.listener.as_raw_fd();
        if let Err(e) = register_event(kqueue, fd, EVFILT_READ, EV_ADD | EV_ENABLE) {
            panic!("[!] Failed to register listener {}: {}", fd, e);
        }
    }
}

/// Process a single kqueue event
pub fn process_event(server: &mut Server, kqueue: RawFd, ev: &kevent64_s) {
    let fd = ev.ident as i32;

    // 1. Accept new connections if this is a listener socket
    if let Some(socket) = server.sockets.iter().find(|s| s.listener.as_raw_fd() == fd) {
        let new_clients = socket.try_accept();
        for mut client in new_clients {
            let cfd = client.stream.as_raw_fd();
            if let Err(e) = register_event(kqueue, cfd, EVFILT_READ, EV_ADD | EV_ENABLE) {
                eprintln!("[!] Failed to register client {}: {}", cfd, e);
                continue;
            }
            server.clients.push(client);
        }
        return;
    }

    // 2. Handle existing client
    if let Some(pos) = server.clients.iter().position(|c| c.stream.as_raw_fd() == fd) {
        let mut client = server.clients.swap_remove(pos);
        let keep = server.handle_client(&mut client).unwrap_or(false);

        if keep {
            server.clients.push(client);
        } else if let Err(e) = register_event(kqueue, fd, EVFILT_READ, EV_DELETE) {
            eprintln!("[!] Failed to deregister {}: {}", fd, e);
        }
    }
}

/// The main server event loop
pub fn run_loop(server: &mut Server) {
    let kqueue = create_kqueue();
    register_listeners(server, kqueue);

    let mut events = vec![
        kevent64_s {
            ident: 0,
            filter: 0,
            flags: 0,
            fflags: 0,
            data: 0,
            udata: 0,
            ext: [0, 0],
        };
        1024
    ];

    loop {
        let nev = unsafe {
            kevent(
                kqueue,
                std::ptr::null(),
                0,
                events.as_mut_ptr() as *mut _,
                events.len() as i32,
                std::ptr::null(),
            )
        };

        if nev < 0 {
            eprintln!("[!] kqueue wait failed: {}", std::io::Error::last_os_error());
            continue;
        }

        for i in 0..nev as usize {
            process_event(server, kqueue, &events[i]);
        }
    }
}
