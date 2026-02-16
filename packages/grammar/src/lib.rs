use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "urd_schema_markdown.pest"]
pub struct UrdParser;

/// Parse a `.urd.md` source string and return the parse tree or an error.
pub fn parse(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    UrdParser::parse(Rule::File, input)
}
