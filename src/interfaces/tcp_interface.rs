use crate::protobuf;
use crate::protobuf::{tcp_message, DataMessage, ProtoVariable, StartMessage, TcpMessage, VariableDescription};
use crate::types::runtime::RuntimeCommand;
use crate::types::EntityVariableKey;
use anyhow::{bail, Context};
use prost::Message;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use crate::interfaces::CommIds;
use crate::protobuf::variable_description::DataType;
use crate::types::value::Value;

pub struct TcpInterface {
    #[allow(unused)]
    listener: TcpListener,
    stream: TcpStream,
    comm_ids: CommIds,
    command_descriptions: HashMap<String, VariableDescription>,
}

impl TcpInterface {
    pub fn connect() -> anyhow::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        log::info!("Listening on {}", listener.local_addr()?);
        let stream = listener.incoming().next().unwrap()?;
        let mut tcp_interface = Self {
            listener,
            stream,
            comm_ids: CommIds::new(),
            command_descriptions: HashMap::new(),
        };
        log::info!("Connected, waiting for setup message");
        tcp_interface.handle_setup_message()?;
        log::debug!("Sending start message");
        tcp_interface.send_start_message()?;

        Ok(tcp_interface)
    }

    pub fn update_variables(&mut self) -> HashMap<EntityVariableKey, Value> {
        let message = self.listen_for_message()
            .expect("Failed to receive variables from controller")
            .expect("Receiving variables timed out");
        let tcp_message::Message::DataMessage(dm) = message.message.unwrap() else {
          panic!("Received invalid message from controller");
        };
        let variables = dm.variables.into_iter().map(|v| {
            let desc = v.meta_data.as_ref().unwrap();

            (EntityVariableKey::new(self.comm_ids.get_name(desc.entity_id), self.comm_ids.get_name(desc.id)), decode_runtime_value(&v, &self.comm_ids))
        }).collect();

        variables
    }

    pub fn execute_command(&mut self, command: &RuntimeCommand) -> anyhow::Result<()> {
        let command_desc = self.command_descriptions.get(&command.name).context("Command has not been registered by controller")?;

        self.send_tcp_message(&TcpMessage {
            message_type: tcp_message::Type::Data as i32,
            timestamp: 0,
            message: Some(tcp_message::Message::DataMessage(DataMessage {
                variables: vec![
                    ProtoVariable {
                        meta_data: Some(command_desc.clone()),
                        data: values_to_le_bytes(&command.params, &self.comm_ids),
                    }
                ],
                time_span: 0,
            })),
        })?;

        Ok(())
    }

    fn handle_setup_message(&mut self) -> anyhow::Result<()> {
        let message = self.listen_for_message()?.unwrap();
        let Some(tcp_message::Message::SetupMessage(setup_message)) = message.message else {
            bail!("Invalid setup message");
        };
        self.comm_ids.insert_map(&setup_message.entities);
        self.comm_ids.insert_map(&setup_message.objects);
        self.comm_ids.insert_map(&setup_message.commands);
        self.command_descriptions = setup_message.command_descriptions.into_iter()
            .map(|c| {
                (c.name, c.description.unwrap())
            }).collect();

        Ok(())
    }

    fn send_start_message(&mut self) -> anyhow::Result<()> {
        self.send_tcp_message(&TcpMessage {
            message_type: tcp_message::Type::Start as i32,
            timestamp: 0,
            message: Some(tcp_message::Message::StartMessage(StartMessage {
                diagnostic_mode: true,
                reconnection_type: 0,
            })),
        })?;

        Ok(())
    }

    fn send_tcp_message(&mut self, message: &TcpMessage) -> anyhow::Result<()> {
        let encoded = message.encode_to_vec();
        let size_bytes = (encoded.len() as u64).to_le_bytes();
        self.stream.write(&size_bytes)?;
        self.stream.write(&encoded)?;

        Ok(())
    }

    fn listen_for_message(&mut self) -> anyhow::Result<Option<TcpMessage>> {
        let mut size_buf = vec![0; 8];
        match self.stream.read_exact(&mut size_buf[..]) {
            Ok(()) => {}
            Err(e) => {
                return if e.kind() == std::io::ErrorKind::TimedOut {
                    Ok(None)
                } else {
                    Err(e.into())
                };
            }
        }
        let size = le_bytes_to_u64(&size_buf[..]);

        let mut data_buf = vec![0; size as usize];
        self.stream.read_exact(&mut data_buf[..])?;

        Ok(Some(protobuf::TcpMessage::decode(data_buf.as_slice())?))
    }
}

fn values_to_le_bytes(values: &[Value], comm_ids: &CommIds) -> Vec<u8> {
    values.into_iter().flat_map(|v| match v {
        Value::Number(v) => v.to_le_bytes().to_vec(),
        // Std should probably never be sent to the controller
        Value::UncertainNumber(m, _) => m.to_le_bytes().to_vec(),
        Value::String(v) => v.as_bytes().to_vec(),
        Value::EntityId(e) => comm_ids.get_id(e).to_le_bytes().to_vec(),
        Value::List(list) => values_to_le_bytes(list, comm_ids),
    }).collect()
}

fn decode_runtime_value(proto_variable: &ProtoVariable, comm_ids: &CommIds) -> Value {
    let meta_data = proto_variable.meta_data.as_ref().unwrap();
    if meta_data.data_type == DataType::Double as i32 {
        if meta_data.dimensions[0] > 1 {
            Value::List(proto_variable.data.chunks(8).map(|d| Value::Number(le_bytes_to_f64(d))).collect())
        }
        else {
            Value::Number(le_bytes_to_f64(&proto_variable.data))
        }
    }
    else if meta_data.data_type == DataType::CommunicationId as i32 {
        let id = le_bytes_to_i64(&proto_variable.data) as i32;
        if id != -1 {
            Value::EntityId(comm_ids.get_name(id).to_owned())
        }
        else {
            Value::List(vec![])
        }
    }
    else if meta_data.data_type == DataType::String as i32 {
        Value::String(le_bytes_to_string(&proto_variable.data))
    }
    else {
        panic!("Unsupported data type received")
    }
}

fn le_bytes_to_string(slice: &[u8]) -> String {
    String::from_utf8(slice.to_vec()).expect("Failed to decode invalid UTF-8 string")
}

fn le_bytes_to_f64(slice: &[u8]) -> f64 {
    let bytes: [u8; 8] = slice.try_into().expect("Incorrect slice length");
    f64::from_le_bytes(bytes)
}

fn le_bytes_to_u64(slice: &[u8]) -> u64 {
    let bytes: [u8; 8] = slice.try_into().expect("Incorrect slice length");
    u64::from_le_bytes(bytes)
}

fn le_bytes_to_i64(slice: &[u8]) -> i64 {
    let bytes: [u8; 8] = slice.try_into().expect("Incorrect slice length");
    i64::from_le_bytes(bytes)
}
