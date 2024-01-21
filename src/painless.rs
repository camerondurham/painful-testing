/// Test case definition for painless script unit test
#[derive(Debug)]
pub struct TestCase<'a> {
    pub id: &'a str,
    pub state: Option<DocRef>,
    pub incoming: DocRef,
    pub expected: Option<DocRef>,
}

/// Document reference
#[derive(Debug)]
pub enum DocRef {
    Filepath(String),
    Raw(String),
}
