use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum DiagType {
    Warning,
    Error,
}

impl Default for DiagType {
    fn default() -> Self {
        DiagType::Warning
    }
}

#[derive(Default)]
pub struct DiagMsg {
    pub diag_type: DiagType,
    pub module: String,
    pub msg: String,
}

impl Display for DiagMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.diag_type {
            DiagType::Error => write!(f, "Error: ({}) {}", self.module, self.msg),
            DiagType::Warning => write!(f, "Warning: ({}) {}", self.module, self.msg)
        }
    }
}

#[derive(Default)]
pub struct Diagnostics {
    pub messages: Vec<DiagMsg>,
    pub warnings: usize,
    pub errors: usize,
}

impl Diagnostics {
    pub fn add_error(&mut self, module: &str, msg: String) {
        self.add_msg(DiagType::Error, module, msg);
    }

    pub fn add_warning(&mut self, module: &str, msg: String) {
        self.add_msg(DiagType::Warning, module, msg);
    }

    pub fn add_msg(&mut self, dtype: DiagType, module: &str, msg: String) {
        match dtype {
            DiagType::Error => self.errors += 1,
            DiagType::Warning => self.warnings += 1,
        }
        let d = DiagMsg {
            diag_type: dtype,
            module: module.to_owned(),
            msg,
        };
        self.messages.push(d);
    }
}
