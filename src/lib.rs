#[derive(Debug)]
pub struct CommitMessage {
    pub status: MessageType,
    pub content: Vec<String>,
}

const MISSIONID: &str = "请输入任务ID:";
const CHANGETYPE: &str = "请选择变更类型:";
const SCOPE: &str = "请输入变更范围:";
const SUBJECT: &str = "请入变更概述:";


#[derive(Debug)]
pub enum MessageType {
    Missionid(& 'static str),
    ChangeType(& 'static str),
    Scope(& 'static str),
    Subject(& 'static str),
}

impl MessageType {
    pub fn content(&self) -> & 'static str {
        match &self {
            MessageType::Missionid(content) => content,
            MessageType::ChangeType(content) => content,
            MessageType::Scope(content) => content,
            MessageType::Subject(content) => content,
        }
    }
}

impl CommitMessage {

    pub fn new() -> CommitMessage {
        CommitMessage {
            status: MessageType::Missionid(MISSIONID),
            content: vec![]
        }
    }

    pub fn status(&self) -> &MessageType {
        &self.status
    }

    pub fn next(& mut self){
        self.status = match self.status {
            MessageType::Missionid(_) => MessageType::ChangeType(CHANGETYPE),
            MessageType::ChangeType(_) => MessageType::Scope(SCOPE),
            MessageType::Scope(_) => MessageType::Subject(SUBJECT),
            MessageType::Subject(s) => MessageType::Subject(s),
        };
    }

    pub fn add_msg(&mut self,s: &str) {
        self.content.push(s.to_string());
    }

    pub fn content(&self) -> String {
        self.content.join(" ")
    }
}