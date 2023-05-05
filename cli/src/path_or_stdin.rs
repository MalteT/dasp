use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    str::FromStr,
};

use fallible_iterator::FallibleIterator;

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub enum PathOrStdin {
    Path(PathBuf),
    Stdin,
}

impl PathOrStdin {
    /// Read either the underlying path or stdin line by line
    pub fn lines(&self) -> Result<impl FallibleIterator<Item = String, Error = Error>> {
        let raw: Box<dyn Iterator<Item = Result<String, Error>>> = match self {
            PathOrStdin::Path(path) => {
                let lines = BufReader::new(File::open(path)?)
                    .lines()
                    .map(|res| res.map_err(Error::from));
                Box::from(lines)
            }
            PathOrStdin::Stdin => {
                let lines = ::std::io::stdin()
                    .lines()
                    .map(|res| res.map_err(Error::from));
                Box::from(lines)
            }
        };
        Ok(fallible_iterator::convert(raw)
            .inspect(|line| {
                log::trace!("Found line: {line:?}");
                Ok(())
            })
            .map(|line| Ok(line.trim().to_owned()))
            .take_while(|line| Ok(!line.is_empty()))
            .inspect(|line| Ok(log::info!("Found update line: {line:?}"))))
    }
}

impl FromStr for PathOrStdin {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(Self::Stdin)
        } else {
            let path: Result<PathBuf, std::convert::Infallible> = s.parse();
            Ok(Self::Path(path.unwrap()))
        }
    }
}

impl std::fmt::Display for PathOrStdin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathOrStdin::Path(path) => write!(f, "{}", path.to_string_lossy()),
            PathOrStdin::Stdin => write!(f, "-"),
        }
    }
}
