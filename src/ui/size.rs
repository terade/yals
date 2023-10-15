use std::fmt::{Display, Error, Formatter};

#[derive(Debug)]
pub struct PrettySize(u64, PrettyStyle);

impl PrettySize {
    pub fn from_bytes(bytes: u64) -> Self {
        Self(bytes, PrettyStyle::NonHumanReadable)
    }
    pub fn from_bytes_with_style(bytes: u64, human_readable: bool) -> Self {
        let style = if human_readable {
            PrettyStyle::HumanReadable
        } else {
            PrettyStyle::NonHumanReadable
        };

        Self(bytes, style)
    }

    pub fn bytes(&self) -> u64 {
        self.0
    }
    fn convert(&self) -> Unit {
        let mut last = Unit::Bytes(self.0 as f64);
        let mut current = Unit::Bytes(self.0 as f64);

        loop {
            last = current;
            current = match current {
                Unit::Bytes(size) => {
                    if (size / 1000_f64) > 1_f64 {
                        Unit::KibiByte((size / 1000_f64).round())
                    } else {
                        current
                    }
                }
                Unit::KibiByte(size) => {
                    if (size / 1000_f64) > 1_f64 {
                        Unit::MegiByte((size / 1000_f64).round())
                    } else {
                        current
                    }
                }
                Unit::MegiByte(size) => {
                    if (size / 1000_f64) > 1_f64 {
                        Unit::GibyByte((size / 1000_f64).round())
                    } else {
                        current
                    }
                }
                Unit::GibyByte(size) => {
                    if (size / 1000_f64) > 1_f64 {
                        Unit::TebiByte((size / 1000_f64).round())
                    } else {
                        current
                    }
                }
                _ => current,
            };

            // if nothing changed conversion finished
            if current == last {
                break;
            }
        }

        current
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Unit {
    Bytes(f64),
    KibiByte(f64),
    MegiByte(f64),
    GibyByte(f64),
    TebiByte(f64),
}

#[derive(Debug, Copy, Clone)]
enum PrettyStyle {
    HumanReadable,
    NonHumanReadable,
}

impl Display for PrettySize {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let unit = self.convert();

        let (size, unit_symbol) = match unit {
            Unit::Bytes(size) => (size, "B"),
            Unit::KibiByte(size) => (size, "K"),
            Unit::MegiByte(size) => (size, "M"),
            Unit::GibyByte(size) => (size, "G"),
            Unit::TebiByte(size) => (size, "T"),
        };

        match self.1 {
            PrettyStyle::HumanReadable => {
                write!(f, "{}{}", size, unit_symbol)
            }
            PrettyStyle::NonHumanReadable => write!(f, "{}", size),
        }
    }
}

#[cfg(test)]
mod test {

    use super::{PrettySize, Unit};
    #[test]
    fn test_size() {
        let size = PrettySize::from_bytes_with_style(19762, true);

        assert_eq!(size.convert(), Unit::KibiByte(20.0));
        assert_eq!(format!("{}", size), String::from("20K"));

        let size = PrettySize::from_bytes(19762000);

        assert_eq!(size.convert(), Unit::MegiByte(20.0));
    }
}
