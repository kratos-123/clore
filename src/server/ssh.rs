use std::collections::HashMap;
use std::io::Read;
use std::net::SocketAddr;
use std::net::TcpStream;

use hickory_resolver::config::*;
use hickory_resolver::TokioAsyncResolver;
use ssh2::Session;
use tracing::info;
use tracing::warn;

use crate::server::address::Deployed;
use crate::server::clore::Clore;

use super::clore::model::my_orders::Order;

pub struct Ssh {}

impl Ssh {
    pub async fn try_run_command_remote(
        orders: &Vec<Order>,
    ) -> (HashMap<String, Deployed>, Vec<u32>) {
        let config = Clore::get_config().await;
        let mut address = HashMap::<String, Deployed>::new();
        let mut errors = Vec::new();
        for order in orders.iter() {
            let domain = order.get_ssh_host();
            let port = order.get_map_ssh_port();

            if domain.is_none() || port.is_none() {
                errors.push(order.order_id);
                warn!("server_id:{}无法进行远程链接", order.server_id);
                continue;
            }

            let sshaddr = domain.unwrap();
            let sshport = port.unwrap();
            info!(
                "远程测试中:server_id:{},order_id:{},{}:{}",
                order.server_id, order.order_id, sshaddr, sshport
            );
            let result = Ssh::get_remote_ip(sshaddr.clone(), sshport).await;
            if result.is_ok() {
                let result = Ssh::exec_to_remote(
                    config.ssh_passwd.clone(),
                    result.unwrap(),
                    "ps -aeo command |grep execute.py |grep -v grep",
                );
                if let Ok(deployed_addr) = result {
                    for addr in deployed_addr.iter() {
                        address.insert(
                            addr.clone(),
                            Deployed::DEPLOYED {
                                orderid: order.order_id,
                                serverid: order.server_id,
                                sshaddr: Some(sshaddr.clone()),
                                sshport: Some(sshport),
                            },
                        );
                    }
                } else {
                    errors.push(order.order_id);
                    warn!("ssh远程操作失败:{:?}", result)
                }
            } else {
                errors.push(order.order_id);
                warn!("ssh远程操作失败:{:?}", result)
            }
        }
        info!("远端总在跑的地址:{:?}", address);
        (address, errors)
    }

    pub fn exec_to_remote(
        ssh_passwd: String,
        socket_addr: SocketAddr,
        ssh_command: &str,
    ) -> Result<Vec<String>, String> {
        let mut address = Vec::new();
        info!("链接远程:{},运行命令:{}", socket_addr, ssh_command);
        let tcp = TcpStream::connect(socket_addr).map_err(|e| e.to_string())?;

        let mut sess = Session::new().map_err(|e| e.to_string())?;
        sess.set_tcp_stream(tcp);
        sess.handshake().map_err(|e| e.to_string())?;
        sess.userauth_password("root", &ssh_passwd)
            .map_err(|e| e.to_string())?;

        let mut channel = sess.channel_session().map_err(|e| e.to_string())?;
        channel.exec(ssh_command).map_err(|e| e.to_string())?;
        let mut output = String::new();

        let result = channel
            .read_to_string(&mut output)
            .map_err(|e| e.to_string());
        let _ = channel.wait_close();
        if result.is_ok() {
            info!("ssh运行结果:\n{}", output);
            let reg: regex::Regex = regex::Regex::new(r"(nimble[\w]+)").unwrap();
            for row in output.split("\n").into_iter() {
                if let Some(captures) = reg.captures(row) {
                    let (_, [addr]) = captures.extract::<1>();
                    let addr = addr.trim().to_string();
                    if !addr.is_empty() {
                        address.push(addr);
                    }
                };
            }
            info!("解析远程地址:{:?}", address);
            if address.is_empty() {
                Err("远程进程无结果".to_string())
            } else {
                Ok(address)
            }
        } else {
            let e = format!("远程执行ssh读取失败{:?}", result);
            warn!(e);
            Err(e)
        }
    }

    pub async fn get_remote_ip(domain: String, port: u16) -> Result<SocketAddr, String> {
        info!("域名解析中:{}:{}", domain, port);
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
