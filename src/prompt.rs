use crate::message::Message;

pub struct Prompt;

impl Prompt {
    pub const FIRST_PROMPT: &'static str = "You are {}, an {} and {} agent.";
    pub const GET_MESSAGE: &'static str = "\n\n{} -> {} : {}";

    pub fn get_message(message: Message) -> String {
        Self::GET_MESSAGE
            .to_string()
            .replace("{}", &message.sender)
            .replace("{}", &message.recipient)
            .replace("{}", &message.content.to_string())
    }
}
