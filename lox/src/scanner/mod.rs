struct Scanner {
    code: String,
}

impl Scanner {
    pub(crate) fn new(code: String) -> Self {
        Scanner { code }
    }

    pub(crate) fn scan(&self) -> Vec<Token> {
        let mut position = 0_usize;

        let chars = self.code.chars().collect::<Vec<_>>();

        println!("{chars:?}");

        todo!()
    }
}


struct Token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {
        let scanner = Scanner::new("ABCDE".to_string());
        scanner.scan();
    }
}


