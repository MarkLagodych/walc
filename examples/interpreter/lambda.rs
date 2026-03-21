use std::collections::HashMap as Map;

/// An ID that uniquely identifies a variable name string.
type VariableId = usize;
/// Index of an expression in the expression vector.
type ExpressionId = usize;

#[derive(Debug)]
enum Expression {
    Variable {
        id: VariableId,
    },
    /// The next item in the vector is the body of the abstraction.
    Abstraction {
        variable: VariableId,
    },
    /// The next item in the vector is the left side of the application.
    Application {
        right: ExpressionId,
    },
}

#[derive(Default, Debug)]
struct Program {
    /// Indexed by [ExpressionId]
    expressions: Vec<Expression>,
    /// Indexed by [VariableId]
    variable_names: Vec<String>,
}

impl Program {
    fn parse(source: &[u8]) -> Result<Self, String> {
        ProgramParser::new(source).parse()
    }
}

struct ProgramTokenizer<'a> {
    source: &'a [u8],
    pub position: usize,
}

impl ProgramTokenizer<'_> {
    fn new(source: &[u8]) -> ProgramTokenizer<'_> {
        ProgramTokenizer {
            source,
            position: 0,
        }
    }

    fn peek_char(&self) -> Option<u8> {
        self.source.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<u8> {
        let c = self.peek_char();
        if c.is_some() {
            self.position += 1;
        }
        c
    }
}

impl<'a> Iterator for ProgramTokenizer<'a> {
    type Item = Result<&'a [u8], String>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.peek_char() {
            match c {
                b';' => {
                    self.next_char();
                    while let Some(c) = self.next_char()
                        && c != b'\n'
                    {}
                }
                c if c.is_ascii_whitespace() => {
                    self.next_char();
                    continue;
                }
                _ => break,
            }
        }

        let start_pos = self.position;

        let is_id_char = |c: u8| c.is_ascii_alphanumeric() || c == b'_';

        match self.next_char()? {
            b'(' | b')' | b'[' | b']' => Some(Ok(&self.source[start_pos..start_pos + 1])),

            c if is_id_char(c) => {
                while let Some(c) = self.peek_char()
                    && is_id_char(c)
                {
                    self.next_char();
                }

                Some(Ok(&self.source[start_pos..self.position]))
            }

            c => Some(Err(format!("Unexpected character '{c}' near {start_pos}"))),
        }
    }
}

fn token_to_str(token: &[u8]) -> &str {
    // Tokens are always ASCII-only, so always valid UTF-8
    unsafe { std::str::from_utf8_unchecked(token) }
}

fn token_to_string(token: &[u8]) -> String {
    token_to_str(token).to_string()
}

struct ProgramParser<'a> {
    tokens: ProgramTokenizer<'a>,
    program: Program,
    variable_ids: Map<String, VariableId>,
}

enum ParsingState {
    AnyTerm,
    AbstractionVariable,
    AbstractionEnd,
    /// When starting to parse the right side, the ExpressionId of the right side is updated in the
    /// application expression, and the state is changed to [Self::ApplicationEnd]
    ApplicationRight {
        application_id: ExpressionId,
    },
    ApplicationEnd,
}

impl<'a> ProgramParser<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self {
            tokens: ProgramTokenizer::new(source),
            program: Program::default(),
            variable_ids: Map::new(),
        }
    }

    /// Returns an estimated position, i.e. not "at" but "near"
    fn next_token_position(&self) -> usize {
        self.tokens.position
    }

    fn next_expression_id(&self) -> ExpressionId {
        self.program.expressions.len()
    }

    fn next_token(&mut self) -> Result<&[u8], String> {
        self.tokens
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())?
    }

    fn get_variable_id(&mut self, name: String) -> VariableId {
        if let Some(id) = self.variable_ids.get(&name) {
            *id
        } else {
            let new_id = self.variable_ids.len();
            self.variable_ids.insert(name.clone(), new_id);
            self.program.variable_names.push(name);
            new_id
        }
    }

    fn parse(mut self) -> Result<Program, String> {
        let mut stack = vec![ParsingState::AnyTerm];

        while let Some(state) = stack.pop() {
            match state {
                ParsingState::AnyTerm => self.parse_any_term(&mut stack)?,
                ParsingState::AbstractionVariable => self.parse_abstraction_variable()?,
                ParsingState::AbstractionEnd => self.parse_abstraction_end()?,
                ParsingState::ApplicationRight { application_id } => {
                    self.handle_application_right(application_id)
                }
                ParsingState::ApplicationEnd => self.parse_application_end()?,
            }
        }

        Ok(self.program)
    }

    fn parse_any_term(&mut self, stack: &mut Vec<ParsingState>) -> Result<(), String> {
        let pos = self.next_token_position();
        let token = self.next_token()?;

        match token {
            b"(" => {
                stack.push(ParsingState::ApplicationEnd);
                stack.push(ParsingState::AnyTerm);
                stack.push(ParsingState::ApplicationRight {
                    application_id: self.next_expression_id(),
                });
                stack.push(ParsingState::AnyTerm);

                self.program
                    .expressions
                    .push(Expression::Application { right: 0 });
            }
            b"[" => {
                stack.push(ParsingState::AbstractionEnd);
                stack.push(ParsingState::AnyTerm);
                stack.push(ParsingState::AbstractionVariable);
            }
            b")" | b"]" => Err(format!(
                "Expected '(', '[', or variable, found '{}' near {}",
                token_to_str(token),
                pos
            ))?,
            variable_token => {
                let variable_name = token_to_string(variable_token);
                let variable_id = self.get_variable_id(variable_name);

                self.program
                    .expressions
                    .push(Expression::Variable { id: variable_id });
            }
        }

        Ok(())
    }

    fn parse_abstraction_variable(&mut self) -> Result<(), String> {
        let pos = self.next_token_position();
        let token = self.next_token()?;

        if let b"(" | b")" | b"[" | b"]" = token {
            Err(format!(
                "Expected variable name, found '{}' near {}",
                token_to_str(token),
                pos
            ))?
        }

        let variable_name = token_to_string(token);
        let variable_id = self.get_variable_id(variable_name);

        self.program.expressions.push(Expression::Abstraction {
            variable: variable_id,
        });

        Ok(())
    }

    fn parse_abstraction_end(&mut self) -> Result<(), String> {
        let pos = self.next_token_position();
        let token = self.next_token()?;

        if token != b"]" {
            Err(format!(
                "Expected ']', found '{}' near {}",
                token_to_str(token),
                pos
            ))?
        }

        Ok(())
    }

    fn handle_application_right(&mut self, application_id: ExpressionId) {
        self.program.expressions[application_id] = Expression::Application {
            right: self.next_expression_id(),
        };
    }

    fn parse_application_end(&mut self) -> Result<(), String> {
        let pos = self.next_token_position();
        let token = self.next_token()?;

        if token != b")" {
            Err(format!(
                "Expected ')', found '{}' near {}",
                token_to_str(token),
                pos
            ))?
        }

        Ok(())
    }
}

fn main() {
    let source = b"[f ([x (f (x x))] [x (f (x x))])]";
    match Program::parse(source) {
        Ok(program) => println!("Parsed program: {:#?}", program),
        Err(e) => eprintln!("Error parsing program: {e}"),
    }
}
