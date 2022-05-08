// Either use nom or chumsky to parse text
// Lets go with chumsky for now

use chumsky::{prelude::*, Parser};
use structs::PspLibProblem;
use thiserror::Error;

pub mod structs;

#[derive(Debug, Error)]
pub enum PspLibParseError {
    #[error("ParseError occurred")]
    ParseError(Vec<Simple<char>>),
    #[error("Project info incomplete")]
    ProjectInfoIncomplete,
}

pub fn parse_psp_lib(content: &str) -> Result<PspLibProblem, PspLibParseError> {
    let file_metadata_parser = crate::file_metadata_parser();
    let metadata_parser = crate::metadata_parser();
    let project_info_parser = crate::project_info_parser();
    let parser = file_metadata_parser
        .then(metadata_parser)
        .then(project_info_parser);

    let (((file_with_basedata, initial_rng), metadata), project_info) = parser
        .parse(content)
        .map_err(PspLibParseError::ParseError)?;

    let project_info: Vec<structs::PspLibProjectInformation> = {
        let mut info = vec![];

        for project_info in project_info {
            info.push(structs::PspLibProjectInformation {
                number: project_info[0],
                jobs: project_info[1],
                relative_date: project_info[2],
                due_date: project_info[3],
                tard_cost: project_info[4],
                mpm_time: project_info[5],
            });
        }

        info
    };

    Ok(PspLibProblem {
        file_with_basedata,
        initial_rng,
        projects: metadata[0],
        jobs: metadata[1],
        horizon: metadata[2],
        resources: structs::PspLibProblemResources {
            renewable: metadata[3],
            nonrenewable: metadata[4],
            doubly_constrained: metadata[5],
        },
        project_info,
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
        .labelled("separator");

    let descriptor = |id| {
        just(id)
            .padded()
            .then_ignore(just(':'))
            .padded()
            .ignore_then(text::int(10))
            .then_ignore(just(' ').repeated().then_ignore(text::ident()).or_not())
            .from_str::<usize>()
            .unwrapped()
            .labelled(id)
    };

    let projects = descriptor("projects");
    let jobs = descriptor("jobs (incl. supersource/sink )");
    let horizon = descriptor("horizon");
    let renewable = descriptor("- renewable");
    let nonrenewable = descriptor("- nonrenewable");
    let doubly_constrained = descriptor("- doubly constrained");

    separator
        .padded()
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

pub(crate) fn project_info_parser() -> impl Parser<char, Vec<Vec<u8>>, Error = Simple<char>> {
    let separator = filter(|c: &char| *c == '*')
        .repeated()
        .ignored()
        .padded()
        .labelled("separator");

    let info = text::int(10)
        .from_str::<u8>()
        .unwrapped()
        .then_ignore(just(' ').repeated())
        .repeated()
        .at_least(6);

    separator
        .then_ignore(just("PROJECT INFORMATION:"))
        .padded()
        .then_ignore(just("pronr.  #jobs rel.date duedate tardcost  MPM-Time"))
        .padded()
        .ignore_then(
            text::whitespace()
                .ignore_then(info)
                .then_ignore(text::newline())
                .repeated(),
        )
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

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
    fn project_info_parser() {
        let file_metadata_parser = crate::file_metadata_parser();
        let metadata_parser = crate::metadata_parser();
        let project_info_parser = crate::project_info_parser();
        let project_info_parser = file_metadata_parser
            .ignore_then(metadata_parser)
            .ignore_then(project_info_parser);

        let meta = project_info_parser.parse(TEST_FILE);
        dbg!(&meta);
        assert!(meta.is_ok());
    }

    #[test]
    fn full_raw_parsing() {
        let file_metadata_parser = crate::file_metadata_parser();
        let metadata_parser = crate::metadata_parser();
        let parser = file_metadata_parser.then(metadata_parser);
        let parser = parser.then(crate::project_info_parser());

        let meta = parser.parse(TEST_FILE);
        dbg!(&meta);
        assert!(meta.is_ok());
    }

    #[test]
    fn parse_psp_lib_test() {
        let output = parse_psp_lib(TEST_FILE);

        dbg!(&output);
        assert!(output.is_ok());
    }

    #[test]
    fn separator_parsing_fail() {
        let content = "asd";

        let output = parse_psp_lib(content);

        assert!(output.is_err());
    }
}
