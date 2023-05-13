mod libssh;

use libssh::SSHSession;
use libssh::PubKey;
use libssh::ssh_keytypes as keytypes;
use libssh::ssh_auth as auth;

fn main() {
    let session = SSHSession::new();
    session.options_set_host("localhost");
    session.options_set_user("baricus");
    session.options_set_port_str("22");

    let session = session.connect().expect("Could not connect");
    let b = session
        .get_server_banner()
        .expect("Cannot get banner, Null");

    println!("{}", b);

    // import a public key
    let key = PubKey::from_base64("AAAAB3NzaC1yc2EAAAADAQABAAABgQDEn4mWo59WJXmVkXlDyDBTkeHHjssZupD42hoZROs2ez4MUYaKEiiPUN98D/331NmidrVwu+P73K4Mo7B8uJpQ9umvx6L4Duw4msQwOSeW9fcaIOnFXhq55WhWBJxv4KAMXROhadr1MWutPSlIVe6M0z//dxXeqOYpH7DZcQVbJECSChJ/BWeC9HYDZJnvQuSa2a3pGaWWVOU1Xr5IHyGMQ3Sxn5JlX8VkK+OH9a/A9n90oS9Q2RytNWdX9SUXV5m1K7SN/Ry7ewfaNg/cKEnpafffGjnvz1YW9CljCQwXD74z74kaIWpATFXH9adxZHzpvOrF85TJDF9btewsRwVN23Jsy7V04Ei/XD33IvXCXjEI2bENf0+vOrSFQA5wE/Y/jueTvJ95q5ouzo1c7nvY9m9LMpSicXyOgbRLmClum940t2mh4wmST1vHXObWTbkEgzflacXmY5SWtpazdKQI2KciDF3OCm4QsLqm2M0aqYQ3PUjSdTKUZIAbXpmus+U=", keytypes::SSH_KEYTYPE_RSA)
        .expect("Invalid key");

    // try it on the server
    match session.userauth_try_publickey(&key) {
        auth::SSH_AUTH_DENIED  => println!("Key not valid"),
        auth::SSH_AUTH_SUCCESS => println!("Key is valid"),

        auth::SSH_AUTH_ERROR => panic!("Serious SSH Error happened!"),
        _ => panic!("Impossible return value from try_publickey"),
    }
    
    session.silent_disconnect();
}
