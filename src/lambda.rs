pub enum Lambda {
    Variable {
        name: String,
    },
    Abstraction {
        variable: String,
        body: Box<Lambda>,
    },
    Apply {
        left: Box<Lambda>,
        right: Box<Lambda>,
    },
}

impl std::fmt::Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lambda::Variable { name } => write!(f, "{}", name),
            Lambda::Abstraction { variable, body } => {
                write!(f, "[{} {}]", variable, body)
            }
            Lambda::Apply { left, right } => {
                write!(f, "({} {})", left, right)
            }
        }
    }
}
