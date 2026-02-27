use super::*;

/// A chain of instructions.
/// This is similar to (linked) list (see [`list::List`]), so e.g.
/// `(Instr1 (Instr2 (Instr3 unreachable)))` is a valid chain of simple instructions.
///
/// However, instruction chains get more complicated when it comes to control instructions.
/// For them we need special "labels" that point to specific segments of the chain, so that
/// control instructions can "jump" to them.
///
/// This is done by constructing the chain in parts: each segment is assigned to a variable
/// (whose name is practically a label) and is used as a tail for the next segment.
///
/// For example, consider the following instructions:
/// ```wat
/// block
///     i32.eqz
///     br_if 0     ;; refers to label X
///     call $foo
/// end             ;; label X
/// call $bar
/// ;;(unreachable) ;; label Y
/// ```
/// Note that labels are indexed relatively and refer to block nesting depth rather than
/// concrete labels.
///
/// The corresponding instruction chain will look like this:
/// ```text
/// let labelY = unreachable in
/// let labelX = (end (call<bar> labelY)) in
/// (block (i32_eqz (br_if<labelX> (call<foo> labelX))))
/// ```
///
/// The resulting `br_if` instruction will jump either to the next instruction (i.e. `call<foo>`)
/// or to `labelX`.
pub type Code = Expr;

enum ChainItem {
    Instruction(instruction::Instruction),
    Label(Code),
}

#[derive(Default)]
pub struct CodeBuilder {
    items: Vec<ChainItem>,
    next_label_id: u32,
}

impl CodeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Code {
        let mut defs = LetExprBuilder::new();
        let mut chain = unreachable();

        for item in self.items.into_iter().rev() {
            match item {
                ChainItem::Instruction(instr) => chain = apply(instr, [chain]),
                ChainItem::Label(label) => {
                    defs.def(label.clone(), chain);
                    chain = var(label);
                }
            }
        }

        defs.build_in(chain)
    }

    pub fn push(&mut self, instruction: instruction::Instruction) {
        self.items.push(ChainItem::Instruction(instruction));
    }

    /// Call this before adding an instruction to make the label point to it.
    pub fn push_label(&mut self, label: Code) {
        self.items.push(ChainItem::Label(label));
    }

    /// Returns a variable that will potentially point to a subsegment of the chain.
    /// To actually generate a label, call `add_label` with the returned variable.
    pub fn make_label(&mut self) -> Code {
        let label = format!("_{}", self.next_label_id);
        self.next_label_id += 1;
        var(label)
    }
}
