use std::fmt::Display;

#[derive(Debug, Eq, PartialEq)]
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
            write!(f, " ({})", module)?;
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
    use super::*;

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", DiagType::Warning), "Warning");
    }

    #[test]
    fn add_and_print() {
        let mut d = Diagnostics::default();
        d.add_warning(Some("module"), "msg".to_owned());
        d.add_error(Some("module"), "errmsg".to_owned());
        d.add_warning(None, "msg2".to_owned());
        assert_eq!(
            format!("{}", d.messages.get(0).unwrap()),
            "Warning: (module) msg".to_owned()
        );
        assert_eq!(
            format!("{}", d.messages.get(1).unwrap()),
            "Error: (module) errmsg".to_owned()
        );
        assert_eq!(
            format!("{}", d.messages.get(2).unwrap()),
            "Warning: msg2".to_owned()
        );
    }
}
