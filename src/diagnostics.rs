use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum DiagType {
    Warning,
    Error,
}

pub struct DiagMsg {
    pub diag_type: DiagType,
    pub module: Option<String>,
    pub msg: String,
}

impl Display for DiagMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.diag_type {
            DiagType::Error => write!(f, "Error:")?,
            DiagType::Warning => write!(f, "Warning:")?,
        }
        if let Some(module) = &self.module {
            write!(f, " ({}) ", module)?;
        }
        write!(f, " {}", self.msg)
    }
}

#[derive(Default)]
pub struct Diagnostics {
    pub messages: Vec<DiagMsg>,
    pub warnings: usize,
    pub errors: usize,
}

impl Diagnostics {
    pub fn add_error(&mut self, module: Option<&str>, msg: String) {
        self.add_msg(DiagType::Error, module, msg);
    }

    pub fn add_warning(&mut self, module: Option<&str>, msg: String) {
        self.add_msg(DiagType::Warning, module, msg);
    }

    pub fn add_msg(&mut self, dtype: DiagType, module: Option<&str>, msg: String) {
        match dtype {
            DiagType::Error => self.errors += 1,
            DiagType::Warning => self.warnings += 1,
        }
        let d = DiagMsg {
            diag_type: dtype,
            module: module.map(|m| m.to_owned()),
            msg,
        };
        self.messages.push(d);
    }
}

#[cfg(test)]
mod test {
    use crate::diagnostics::DiagType;

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", DiagType::Warning), "Warning");
    }
}
