// Either use nom or chumsky to parse text
// Lets go with chumsky for now

use chumsky::{prelude::*, Parser};
use structs::PspLibProblem;

pub mod structs;

pub fn parse_psp_lib(content: &str) -> Result<PspLibProblem, Vec<Simple<char>>> {
    let (file_with_basedata, initial_rng) = file_metadata_parser().parse(content)?;

    Ok(PspLibProblem {
        file_with_basedata,
        initial_rng,
        projects: 0,
        jobs: 0,
        horizon: 0,
        resources: structs::PspLibProblemResources {
            renewable: 0,
            nonrenewable: 0,
            doubly_constrained: 0,
        },
        project_info: vec![],
        precedence_relations: vec![],
        request_durations: vec![],
        resource_availabilities: structs::PspLibResourceAvailability {
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
        },
    })
}

pub(crate) fn file_metadata_parser() -> impl Parser<char, (String, usize), Error = Simple<char>> {
    let separator = filter(|c: &char| *c == '*')
        .repeated()
        .ignored()
        .labelled("separator");

    let alphanumeric_with_punctuation =
        filter(|c: &char| c.is_ascii_alphanumeric() || c.is_ascii_punctuation()).repeated();

    let basedata = just("file with basedata")
        .padded()
        .then_ignore(just(':'))
        .padded()
        .ignore_then(alphanumeric_with_punctuation)
        .collect::<String>()
        .labelled("basedata");

    let initial_rng = just("initial value random generator")
        .padded()
        .ignore_then(just(':'))
        .padded()
        .ignore_then(text::int(10))
        .from_str::<usize>()
        .unwrapped()
        .labelled("initial_rng");

    separator
        .padded()
        .ignore_then(basedata)
        .padded()
        .then(initial_rng)
}

pub(crate) fn metadata_parser() -> impl Parser<char, Vec<usize>, Error = Simple<char>> {
    let separator = filter(|c: &char| *c == '*')
        .repeated()
        .ignored()
        .padded()
        .labelled("separator");

    let projects = just("projects")
        .padded()
        .then_ignore(just(':'))
        .padded()
        .ignore_then(text::int(10))
        .from_str::<usize>()
        .unwrapped()
        .labelled("projects");

    let jobs = just("jobs (incl. supersource/sink )")
        .padded()
        .then_ignore(just(':'))
        .padded()
        .ignore_then(text::int(10))
        .from_str::<usize>()
        .unwrapped()
        .labelled("jobs");

    let horizon = just("horizon")
        .padded()
        .then_ignore(just(':'))
        .padded()
        .ignore_then(text::int(10))
        .from_str::<usize>()
        .unwrapped()
        .labelled("horizon");

    let renewable = just("- renewable")
        .padded()
        .then_ignore(just(':'))
        .padded()
        .ignore_then(text::int(10))
        .padded()
        .then_ignore(text::ident())
        .from_str::<usize>()
        .unwrapped()
        .labelled("renewable");
    let nonrenewable = just("- nonrenewable")
        .padded()
        .then_ignore(just(':'))
        .padded()
        .ignore_then(text::int(10))
        .padded()
        .then_ignore(text::ident())
        .from_str::<usize>()
        .unwrapped()
        .labelled("nonrenewable");
    let doubly_constrained = just("- doubly constrained")
        .padded()
        .then_ignore(just(':'))
        .padded()
        .ignore_then(text::int(10))
        .padded()
        .then_ignore(text::ident())
        .from_str::<usize>()
        .unwrapped()
        .labelled("doubly constrained");

    separator
        .ignore_then(projects)
        .padded()
        .chain(jobs)
        .padded()
        .chain(horizon)
        .padded()
        .then_ignore(just("RESOURCES"))
        .padded()
        .chain(renewable)
        .padded()
        .chain(nonrenewable)
        .padded()
        .chain(doubly_constrained)
        .collect()
}

#[cfg(test)]
mod tests {
    use chumsky::{error::SimpleReason, Parser};

    use crate::parse_psp_lib;

    static TEST_FILE: &str = include_str!("../../examples/j1201_1.sm");

    #[test]
    fn file_metadata_parsing() {
        let file_metadata_parser = crate::file_metadata_parser();

        let file_meta = file_metadata_parser.parse(TEST_FILE);
        assert!(file_meta.is_ok());

        let (file_with_basedata, initial_rng) = file_meta.unwrap();
        assert_eq!(file_with_basedata, "J1201_.BAS");
        assert_eq!(initial_rng, 24418);
    }

    #[test]
    fn metadata_parsing() {
        let file_metadata_parser = crate::file_metadata_parser();
        let metadata_parser = crate::metadata_parser();
        let metadata_parser = file_metadata_parser.ignore_then(metadata_parser);

        let meta = metadata_parser.parse(TEST_FILE);
        dbg!(&meta);
        assert!(meta.is_ok());
    }

    #[test]
    fn full_parsing() {
        let file_metadata_parser = crate::file_metadata_parser();
        let metadata_parser = crate::metadata_parser();
        let metadata_parser = file_metadata_parser.then(metadata_parser);

        let meta = metadata_parser.parse(TEST_FILE);
        dbg!(&meta);
        assert!(meta.is_ok());
    }

    #[test]
    fn separator_parsing_fail() {
        let content = "asd";

        let output = parse_psp_lib(content);

        assert!(output.is_err());

        assert_eq!(
            output.unwrap_err().first().unwrap().reason(),
            &SimpleReason::Unexpected
        );
    }
}
