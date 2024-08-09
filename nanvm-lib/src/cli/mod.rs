use std::collections::HashMap;
use std::env;
use std::result;

pub struct OptionDefinition {
    name: String,
    // TODO: to be specialized
}

pub struct Option {} // TODO: to be specialized

pub struct CommandDefinition {
    name: String,
    description: String,
    option_definitions: Vec<OptionDefinition>,
    handler: fn(Vec<Option>),
}

pub struct ParsedCommand<'a> {
    command_definition: &'a CommandDefinition,
    options: Vec<Option>,
}

pub enum ParseError {
    // TODO: to be specialized (current values are preliminary)
    UnknownCommand,
    MissingOption,
    InvalidOption,
}

pub type ParseResult<'a> = result::Result<(ParsedCommand<'a>, Vec<Option>), ParseError>;

pub struct CLI {
    command_definitions: HashMap<&'static str, CommandDefinition>,
}

impl CLI {
    fn add_command_definition(&mut self, _command: CommandDefinition) {
        // TODO: Implement this method.
    }

    fn parse(&self, _args: Vec<String>) -> ParseResult {
        // TODO: Implement this method.
        Err(ParseError::UnknownCommand)
    }

    pub fn execute(&self) {
        match self.parse(env::args().collect()) {
            Ok((parsed_command, options)) => {
                (parsed_command.command_definition.handler)(options);
            }
            Err(_e) => {
                // TODO: Handle the error case here.
            }
        }
    }
}
