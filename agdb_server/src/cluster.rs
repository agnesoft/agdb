use crate::config::Config;
use crate::db_pool::ServerUser;
use crate::server_error::ServerResult;
use agdb::AgdbSerialize;
use std::collections::VecDeque;

pub(crate) enum Command {
    DbAdd,
    DbBackup,
    DbCopy,
    DbDelete,
    DbExec,
    DbOptimize,
    DbRemove,
    DbRename,
    DbRestore,
    DbUserAdd,
    DbUserRemove,
    UserAdd(ServerUser),
    UserChangePassword,
    UserRemove,
}

pub(crate) struct Cluster {
    commands: VecDeque<Command>,
    nodes: Vec<String>,
    threshold: u8,
    client: reqwest::Client,
}

impl Cluster {
    pub(crate) fn new(config: &Config) -> Self {
        let self_node = format!("{}:{}", config.host, config.port);

        Self {
            commands: VecDeque::new(),
            nodes: config
                .cluster
                .iter()
                .filter_map(|node| {
                    if node != &self_node {
                        Some(format!("{node}/api/v1/cluster"))
                    } else {
                        None
                    }
                })
                .collect(),
            threshold: config.cluster.len() as u8 / 2,
            client: reqwest::Client::new(),
        }
    }

    pub(crate) async fn process(&mut self, command: Command) -> ServerResult<()> {
        let mut tasks = vec![];

        for node in &self.nodes {
            let data = command.serialize();
            tasks.push(tokio::spawn(
                self.client.post(node.clone()).body(data).send(),
            ));
        }

        for task in tasks {
            let response = task.await??;
            let status = response.status();
            let data = response.bytes().await?;
        }

        Ok(())
    }
}

impl AgdbSerialize for ServerUser {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.username.serialize());
        bytes.extend_from_slice(&self.password.serialize());
        bytes.extend_from_slice(&self.salt.serialize());
        bytes.extend_from_slice(&self.token.serialize());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, agdb::DbError> {
        let mut pos = 0;
        let username = String::deserialize(bytes)?;
        pos += username.serialized_size() as usize;
        let password = Vec::deserialize(&bytes[pos..])?;
        pos += password.serialized_size() as usize;
        let salt = Vec::deserialize(&bytes[pos..])?;
        pos += salt.serialized_size() as usize;
        let token = String::deserialize(&bytes[pos..])?;

        Ok(Self {
            db_id: None,
            username,
            password,
            salt,
            token,
        })
    }

    fn serialized_size(&self) -> u64 {
        todo!()
    }
}

impl AgdbSerialize for Command {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Self::DbAdd => vec![0],
            Self::DbBackup => vec![1],
            Self::DbCopy => vec![2],
            Self::DbDelete => vec![3],
            Self::DbExec => vec![4],
            Self::DbOptimize => vec![5],
            Self::DbRemove => vec![6],
            Self::DbRename => vec![7],
            Self::DbRestore => vec![8],
            Self::DbUserAdd => vec![9],
            Self::DbUserRemove => vec![10],
            Self::UserAdd(user) => {
                let mut bytes = vec![11];
                bytes.extend_from_slice(&user.serialize());

                bytes
            }
            Self::UserChangePassword => vec![12],
            Self::UserRemove => vec![13],
        }
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, agdb::DbError> {
        let variant = bytes[0];

        match variant {
            0 => Ok(Self::DbAdd),
            1 => Ok(Self::DbBackup),
            2 => Ok(Self::DbCopy),
            3 => Ok(Self::DbDelete),
            4 => Ok(Self::DbExec),
            5 => Ok(Self::DbOptimize),
            6 => Ok(Self::DbRemove),
            7 => Ok(Self::DbRename),
            8 => Ok(Self::DbRestore),
            9 => Ok(Self::DbUserAdd),
            10 => Ok(Self::DbUserRemove),
            11 => {
                let user = ServerUser::deserialize(&bytes[1..])?;
                Ok(Self::UserAdd(user))
            }
            12 => Ok(Self::UserChangePassword),
            13 => Ok(Self::UserRemove),
            _ => Err(format!("Invalid command type '{variant}'").into()),
        }
    }

    fn serialized_size(&self) -> u64 {
        match self {
            Self::DbAdd => 1,
            Self::DbBackup => 1,
            Self::DbCopy => 1,
            Self::DbDelete => 1,
            Self::DbExec => 1,
            Self::DbOptimize => 1,
            Self::DbRemove => 1,
            Self::DbRename => 1,
            Self::DbRestore => 1,
            Self::DbUserAdd => 1,
            Self::DbUserRemove => 1,
            Self::UserAdd(user) => 1 + user.serialized_size(),
            Self::UserChangePassword => 1,
            Self::UserRemove => 1,
        }
    }
}
