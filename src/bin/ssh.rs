use hickory_resolver::config::*;
use hickory_resolver::Resolver;
use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::*;

fn main() {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

    // Lookup the IP addresses associated with a name.
    // The final dot forces this to be an FQDN, otherwise the search rules as specified
    //  in `ResolverOpts` will take effect. FQDN's are generally cheaper queries.
    let response = resolver.lookup_ip("n3.c1.clorecloud.net").unwrap();

    // There can be many addresses associated with the name,
    //  this can return IPv4 and/or IPv6 addresses
    let address = response.iter().next().expect("no addresses returned!");

    let socket_addr = SocketAddr::from((address, 10610));
    // Connect to the local SSH server
    let tcp = TcpStream::connect(socket_addr).unwrap();

    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("root", "MTcxNjMwNDc2N19ZempBSW")
        .unwrap();

    let mut channel = sess.channel_session().unwrap();
    channel
        .exec("ps -aeo command |grep execute.py |grep -v grep")
        .map_err(|e| e.to_string())
        .unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    println!("{}", s);
    let _ = channel.wait_close();
    println!("{}", channel.exit_status().unwrap());
}
