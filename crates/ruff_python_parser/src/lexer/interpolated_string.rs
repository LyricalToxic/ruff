use ruff_python_ast::StringFlags;

use crate::string::InterpolatedStringKind;

use super::TokenFlags;

/// The context representing the current f-string or t-string that the lexer is in.
#[derive(Clone, Debug)]
pub(crate) struct InterpolatedStringContext {
    flags: TokenFlags,

    /// The level of nesting for the lexer when it entered the current f/t-string.
    /// The nesting level includes all kinds of parentheses i.e., round, square,
    /// and curly.
    nesting: u32,

    /// The current depth of format spec for the current f/t-string. This is because
    /// there can be multiple format specs nested for the same f-string.
    /// For example, `{a:{b:{c}}}` has 3 format specs.
    format_spec_depth: u32,
}

impl InterpolatedStringContext {
    pub(crate) const fn new(flags: TokenFlags, nesting: u32) -> Option<Self> {
        if flags.is_interpolated_string() {
            Some(Self {
                flags,
                nesting,
                format_spec_depth: 0,
            })
        } else {
            None
        }
    }

    pub(crate) fn kind(&self) -> InterpolatedStringKind {
        if self.flags.is_f_string() {
            InterpolatedStringKind::FString
        } else if self.flags.is_t_string() {
            InterpolatedStringKind::TString
        } else {
            unreachable!("Can only be constructed when f-string or t-string flag is present")
        }
    }

    pub(crate) const fn flags(&self) -> TokenFlags {
        self.flags
    }

    pub(crate) const fn nesting(&self) -> u32 {
        self.nesting
    }

    /// Returns the quote character for the current f-string.
    pub(crate) fn quote_char(&self) -> char {
        self.flags.quote_style().as_char()
    }

    /// Returns the triple quotes for the current f-string if it is a triple-quoted
    /// f-string, `None` otherwise.
    pub(crate) fn triple_quotes(&self) -> Option<&'static str> {
        if self.is_triple_quoted() {
            Some(self.flags.quote_str())
        } else {
            None
        }
    }

    /// Returns `true` if the current f-string is a raw f-string.
    pub(crate) fn is_raw_string(&self) -> bool {
        self.flags.is_raw_string()
    }

    /// Returns `true` if the current f-string is a triple-quoted f-string.
    pub(crate) fn is_triple_quoted(&self) -> bool {
        self.flags.is_triple_quoted()
    }

    /// Calculates the number of open parentheses for the current f-string
    /// based on the current level of nesting for the lexer.
    const fn open_parentheses_count(&self, current_nesting: u32) -> u32 {
        current_nesting.saturating_sub(self.nesting)
    }

    /// Returns `true` if the lexer is in an f-string expression or t-string interpolation i.e., between
    /// two curly braces.
    pub(crate) const fn is_in_interpolation(&self, current_nesting: u32) -> bool {
        self.open_parentheses_count(current_nesting) > self.format_spec_depth
    }

    /// Returns `true` if the lexer is in a f-string format spec i.e., after a colon.
    pub(crate) const fn is_in_format_spec(&self, current_nesting: u32) -> bool {
        self.format_spec_depth > 0 && !self.is_in_interpolation(current_nesting)
    }

    /// Returns `true` if the context is in a valid position to start format spec
    /// i.e., at the same level of nesting as the opening parentheses token.
    /// Increments the format spec depth if it is.
    ///
    /// This assumes that the current character for the lexer is a colon (`:`).
    pub(crate) fn try_start_format_spec(&mut self, current_nesting: u32) -> bool {
        if self
            .open_parentheses_count(current_nesting)
            .saturating_sub(self.format_spec_depth)
            == 1
        {
            self.format_spec_depth += 1;
            true
        } else {
            false
        }
    }

    /// Decrements the format spec depth if the current f-string is in a format
    /// spec.
    pub(crate) fn try_end_format_spec(&mut self, current_nesting: u32) {
        if self.is_in_format_spec(current_nesting) {
            self.format_spec_depth = self.format_spec_depth.saturating_sub(1);
        }
    }
}

/// The interpolated strings stack is used to keep track of all the f-strings and t-strings that the
/// lexer encounters. This is necessary because f-strings and t-strings can be nested.
#[derive(Debug, Default)]
pub(crate) struct InterpolatedStrings {
    stack: Vec<InterpolatedStringContext>,
}

impl InterpolatedStrings {
    pub(crate) fn push(&mut self, context: InterpolatedStringContext) {
        self.stack.push(context);
    }

    pub(crate) fn pop(&mut self) -> Option<InterpolatedStringContext> {
        self.stack.pop()
    }

    pub(crate) fn current(&self) -> Option<&InterpolatedStringContext> {
        self.stack.last()
    }

    pub(crate) fn current_mut(&mut self) -> Option<&mut InterpolatedStringContext> {
        self.stack.last_mut()
    }

    pub(crate) fn checkpoint(&self) -> InterpolatedStringsCheckpoint {
        InterpolatedStringsCheckpoint(self.stack.clone())
    }

    pub(crate) fn rewind(&mut self, checkpoint: InterpolatedStringsCheckpoint) {
        self.stack = checkpoint.0;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InterpolatedStringsCheckpoint(Vec<InterpolatedStringContext>);
