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
