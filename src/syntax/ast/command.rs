use crate::{io::FileDescriptor, symbol::Symbol};
use super::{lexer, IllFormed, SourcePos};


/// The most basic part of an argument.
#[derive(Debug)]
pub enum ArgUnit {
	Literal(Box<[u8]>),
	Dollar {
		symbol: Symbol,
		pos: SourcePos,
	}
}


/// The most basic part of an argument.
#[derive(Debug)]
pub enum ArgPart {
	Unit(ArgUnit),

	// Expansions:
	Home, // ~/
	Range(i64, i64), // {x..y}
	Collection(Box<[ArgUnit]>), // {a,b,c}

	// Regex expansions:
	Star, // *
	Question, // ?
	CharClass(Box<[u8]>), // [...]
}


/// An argument may consist of several argument parts.
#[derive(Debug)]
pub struct Argument {
	pub parts: Box<[ArgPart]>,
	pub pos: SourcePos,
}


impl IllFormed for Argument {
	fn ill_formed() -> Self {
		Self {
			parts: Default::default(),
			pos: SourcePos::ill_formed(),
		}
	}

	fn is_ill_formed(&self) -> bool {
		self.pos.is_ill_formed()
	}
}


/// The target of a redirection operation.
#[derive(Debug)]
pub enum RedirectionTarget {
	/// Redirect to a file descriptor.
	Fd(FileDescriptor),
	/// Overwrite a file.
	Overwrite(Argument),
	/// Append to a file.
	Append(Argument),
}


/// Redirection operation.
#[derive(Debug)]
pub enum Redirection {
	/// An ill-formed redirection, produced by a parse error.
	IllFormed,
	/// Redirect output to a file or file descriptor.
	Output {
		source: FileDescriptor,
		target: RedirectionTarget,
	},
	/// Redirect input from a file or literal.
	Input {
		/// Whether the source is the input or the file path.
		literal: bool,
		source: Argument,
	},
}


impl IllFormed for Redirection {
	fn ill_formed() -> Self {
		Self::IllFormed
	}

	fn is_ill_formed(&self) -> bool {
		matches!(self, Self::IllFormed)
	}
}


/// A single command, including possible redirections and try operator.
#[derive(Debug)]
pub struct BasicCommand {
	pub program: Argument,
	pub arguments: Box<[Argument]>,
	pub redirections: Box<[Redirection]>,
	pub abort_on_error: bool,
	pub pos: SourcePos,
}


impl IllFormed for BasicCommand {
	fn ill_formed() -> Self {
		Self {
			program: Argument::ill_formed(),
			arguments: Default::default(),
			redirections: Default::default(),
			abort_on_error: Default::default(),
			pos: SourcePos::ill_formed(),
		}
	}

	fn is_ill_formed(&self) -> bool {
		self.pos.is_ill_formed()
	}
}


/// Commands may be pipelines, or a single BasicCommand.
#[derive(Debug)]
pub struct Command {
	pub head: BasicCommand,
	pub tail: Box<[BasicCommand]>,
}


impl IllFormed for Command {
	fn ill_formed() -> Self {
		Self {
			head: BasicCommand::ill_formed(),
			tail: Default::default(),
		}
	}

	fn is_ill_formed(&self) -> bool {
		self.head.is_ill_formed()
	}
}


/// A command block.
#[derive(Debug)]
pub struct CommandBlock {
	pub kind: CommandBlockKind,
	pub head: Command,
	pub tail: Box<[Command]>,
}


impl IllFormed for CommandBlock {
	fn ill_formed() -> Self {
		Self {
			kind: CommandBlockKind::Synchronous,
			head: Command::ill_formed(),
			tail: Default::default(),
		}
	}

	fn is_ill_formed(&self) -> bool {
		self.head.is_ill_formed()
	}
}


/// The kinds of command blocks.
#[derive(Debug)]
pub enum CommandBlockKind {
	Synchronous,  // {}
	Asynchronous, // &{}
	Capture,      // ${}
}


impl CommandBlockKind {
	pub fn from_token(token: &lexer::TokenKind) -> Option<Self> {
		match token {
			lexer::TokenKind::Command => Some(Self::Synchronous),
			lexer::TokenKind::AsyncCommand => Some(Self::Asynchronous),
			lexer::TokenKind::CaptureCommand => Some(Self::Capture),
			_ => None,
		}
	}


	/// Check whether the command block should be executed synchronously.
	pub fn is_sync(&self) -> bool {
		matches!(self, Self::Synchronous)
	}
}
