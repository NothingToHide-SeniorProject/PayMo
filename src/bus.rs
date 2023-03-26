use log::debug;
use std::path::PathBuf;

use crate::msgs;

pub const CLIENT_PUB_SOCKET: &str = "ipc://{data_dir}/pub-client.ipc";
pub const CLIENT_SUB_SOCKET: &str = "ipc://{data_dir}/sub-client.ipc";

pub fn connect_to_client_sockets(
    data_dir: PathBuf,
    zmq_context: zmq::Context,
    filter: msgs::Process,
) -> crate::Result<(zmq::Socket, zmq::Socket)> {
    let to_client_socket_addr =
        str::replace(CLIENT_SUB_SOCKET, "{data_dir}", data_dir.to_str().unwrap());
    debug!("to_client_socket_addr: {}", to_client_socket_addr);

    let from_client_socket_addr =
        str::replace(CLIENT_PUB_SOCKET, "{data_dir}", data_dir.to_str().unwrap());
    debug!("from_client_socket_addr: {}", from_client_socket_addr);

    let to_client_socket = zmq_context.socket(zmq::PUB)?;
    to_client_socket.connect(&to_client_socket_addr)?;

    let from_client_socket = zmq_context.socket(zmq::SUB)?;
    from_client_socket.connect(&from_client_socket_addr)?;

    from_client_socket.set_subscribe(filter.as_str_name().as_bytes())?;

    Ok((to_client_socket, from_client_socket))
}
