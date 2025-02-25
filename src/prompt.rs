//prompt.rs
use crate::message::Message;
use crate::personality::Personality;

pub struct Prompt;

impl Prompt {
    pub const FIRST_PROMPT: &'static str = "The entrypoint of this world is: {0}.\n\nYou are {1}, you have the following personality : {2}.\
                                            You will be able to interact with other agents in the system. Try to keep your answers short and concise.\
                                            However, you are free, dont feel constrained by the system. You can do whatever you want. \
                                            You can move around the world, talk to other agents, or just observe.";
    pub const GET_MESSAGE: &'static str = "\n\n{1} -> {2} : {3}";

    pub fn get_first_prompt( system_prompt: String,name: String, personality: Personality) -> String {
        Self::FIRST_PROMPT
            .to_string()
            .replace("{0}", system_prompt.as_str())
            .replace("{1}", name.as_str())
            .replace("{2}", personality.get_description())
    }

    pub fn get_message(message: Message) -> String {
        Self::GET_MESSAGE
            .to_string()
            .replace("{1}", &message.sender)
            .replace("{2}", &message.recipient)
            .replace("{3}", &message.content.to_string())
    }
}
