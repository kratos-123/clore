use std::collections::HashMap;
use std::io::Read;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::TcpStream;

use futures::executor::block_on;
use hickory_resolver::config::*;
use hickory_resolver::proto::rr::domain;
use hickory_resolver::TokioAsyncResolver;
use ssh2::Session;
use tracing::info;
use tracing::warn;

use super::clore::model::my_orders;
use super::clore::model::my_orders::MyOrders;

pub struct Ssh {}

impl Ssh {
    pub async fn try_run_command_remote(my_orders: &mut MyOrders) {
        let mut address = HashMap::<i32,Vec<String>>::new();
        for order in (*my_orders).iter_mut() {
            let domain = order.get_ssh_host();
            let port = order.get_map_ssh_port();
            if domain.is_none() || port.is_none() {
                warn!(
                    "无法进行远程链接：server_id:{:?},host:{:?},port:{:?}",
                    order.server_id, domain, port
                );
                continue;
            }
            let domain = domain.unwrap();
            let port = port.unwrap();
            let result = Ssh::get_remote_ip(domain, port).await;
            if result.is_ok() {
                let result = Ssh::exec_to_remote(result.unwrap(), "");
                if result.is_ok() {
                   let addr = result.unwrap();
                   address.insert(order.server_id, addr);
                }
            } else {
                warn!("ssh远程操作失败:{:?}",result)
            }
        }
        info!("远端总在跑的地址:{:?}",address);
    }

    pub fn exec_to_remote(
        socket_addr: SocketAddr,
        ssh_command: &str,
    ) -> Result<Vec<String>, String> {
        let mut address = Vec::new();
        let ssh_command = "ps -aeo command |grep execute.py |grep -v grep";
        info!("链接远程:{},运行命令:{}", socket_addr, ssh_command);
        let tcp = TcpStream::connect(socket_addr).map_err(|e| e.to_string())?;

        let mut sess = Session::new().map_err(|e|e.to_string())?;
        sess.set_tcp_stream(tcp);
        sess.handshake().map_err(|e|e.to_string())?;
        sess.userauth_password("root", "MTcxNjMwNDc2N19ZempBSW").map_err(|e|e.to_string())?;

        let mut channel = sess.channel_session().map_err(|e|e.to_string())?;
        channel.exec(ssh_command).map_err(|e|e.to_string())?;
        let mut output = String::new();
 
        let result = channel
            .read_to_string(&mut output)
            .map_err(|e| e.to_string());
        let _ = channel.wait_close();
        if result.is_ok() {
            info!("ssh运行结果:{}", output);
            for row in output.split("\n").into_iter() {
                let mut addr = row
                    .replace("python execute.py", "")
                    .replace("python3 execute.py", "");
                addr = addr.trim().to_string();
                if !addr.is_empty() {
                    address.push(addr);
                }
            }
            info!("解析远程地址:{:?}", address);
            Ok(address)
        }else {
            let e = format!("远程执行ssh读取失败{:?}",result);
            warn!(e);
            Err(e)
        }
       
    }

    pub async fn get_remote_ip(domain: String, port: u16) -> Result<SocketAddr, String> {
        info!("域名解析中:{}", domain);
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
        let response = resolver
            .lookup_ip(domain)
            .await
            .map_err(|e| e.to_string())?;

        let address = response
            .iter()
            .next()
            .ok_or::<String>("未找到dns记录".to_string())?;

        let socket_addr = SocketAddr::from((address, port));
        info!("域名解析成功:{:?}", socket_addr);
        Ok(socket_addr)
    }
}
