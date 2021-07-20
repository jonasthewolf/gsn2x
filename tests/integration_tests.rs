#[cfg(test)]
mod test {

    use assert_cmd::prelude::*; // Add methods on commands
    use std::{io::Read, process::Command}; // Run programs
    #[test]
    fn example_back_to_back() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("example.gsn.yaml").arg("example.gsn.test.dot");
        cmd.assert().success();
        let mut orig = String::new();
        std::fs::File::open("example.gsn.dot")?.read_to_string(&mut orig)?;
        let mut test = String::new();
        std::fs::File::open("example.gsn.test.dot")?.read_to_string(&mut test)?;
        assert_eq!(orig, test);
        Ok(())
    }
}
