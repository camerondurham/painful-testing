#[derive(Debug)]
pub struct TestCase<'a> {
    pub id: &'a str,
    pub state: Option<DocRef>,
    pub incoming: DocRef,
    pub expected: Option<DocRef>,
}

#[derive(Debug)]
pub enum DocRef {
    Filepath(String),
    Raw(String),
}

pub mod cli;
pub mod docker;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
