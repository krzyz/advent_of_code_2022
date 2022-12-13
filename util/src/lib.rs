use miette::GraphicalReportHandler;
use nom::IResult;
use nom_locate::LocatedSpan;
use nom_supreme::{
    error::{BaseErrorKind, ErrorTree, GenericErrorTree},
    final_parser::final_parser,
};

// Thanks to FasterThanLime! https://fasterthanli.me/series/advent-of-code-2022/part-11

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("bad input")]
struct BadInput<'a> {
    #[source_code]
    src: &'a str,

    #[label("{kind}")]
    bad_bit: miette::SourceSpan,

    kind: BaseErrorKind<&'a str, Box<dyn std::error::Error + Send + Sync>>,
}

pub fn parse_nice<'a, T, F>(l: &'a str, parse_fun: F) -> Option<T>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, T, ErrorTree<Span<'a>>>,
{
    let line_span = Span::new(l);
    let line: Result<_, ErrorTree<Span>> = final_parser(parse_fun)(line_span);
    match line {
        Ok(line) => Some(line),
        Err(e) => {
            match e {
                GenericErrorTree::Base { location, kind } => {
                    let offset = location.location_offset().into();
                    let err = BadInput {
                        src: l,
                        bad_bit: miette::SourceSpan::new(offset, 0.into()),
                        kind,
                    };
                    let mut s = String::new();
                    GraphicalReportHandler::new()
                        .render_report(&mut s, &err)
                        .unwrap();
                    println!("{s}");
                }
                GenericErrorTree::Stack { .. } => todo!("stack"),
                GenericErrorTree::Alt(_) => todo!("alt"),
            }
            None
        }
    }
}
