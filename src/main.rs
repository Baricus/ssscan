mod libssh;

use libssh::SSHSession;

fn main() {
    
    let session = SSHSession::new();
    session.options_set_host("localhost");
    session.options_set_user("baricus");
    session.options_set_port_str("22");

    let session = session.connect().expect("Could not connect");
    let b = session.get_server_banner().expect("Cannot get banner, Null");

    println!("{}", b);
}
