mod operator;
mod semaphore;
pub use operator::{run_operators, Operator, OperatorRole};

#[derive(Debug)]
pub enum OperatorParseError {
    InvalidId,
    InvalidRole,
    InvalidStartAt,
    InvalidDuration,
    TooManyFields,
}

/// Parse an operator from a string.
///
/// # Examples
///
/// ```
/// use ex_2::*;
///
/// assert_eq!(
///     parse_operator("1 R 3.3 5").unwrap(),
///     Operator {
///         id: 1,
///         role: OperatorRole::Reader,
///         start_at: 3.3,
///         duration: 5.,
///     }
/// );
/// ```
pub fn parse_operator(line: &str) -> Result<Operator, OperatorParseError> {
    let mut fields = line.split(' ');

    let id = match fields.next() {
        Some(i) => i.parse().map_err(|_e| OperatorParseError::InvalidId)?,
        None => Err(OperatorParseError::InvalidId)?,
    };

    let role = match fields.next() {
        Some(r) => match r {
            "R" => OperatorRole::Reader,
            "W" => OperatorRole::Writer,
            _ => Err(OperatorParseError::InvalidRole)?,
        },
        None => Err(OperatorParseError::InvalidRole)?,
    };

    let start_at = match fields.next() {
        Some(s) => s.parse().map_err(|_e| OperatorParseError::InvalidRole)?,
        None => Err(OperatorParseError::InvalidStartAt)?,
    };
    if start_at <= 0. {
        Err(OperatorParseError::InvalidStartAt)?
    }

    let duration = match fields.next() {
        Some(d) => d
            .parse()
            .map_err(|_e| OperatorParseError::InvalidDuration)?,
        None => Err(OperatorParseError::InvalidDuration)?,
    };
    if duration <= 0. {
        Err(OperatorParseError::InvalidDuration)?
    }

    if fields.next().is_some() {
        Err(OperatorParseError::TooManyFields)?
    }

    Ok(Operator {
        id,
        role,
        start_at,
        duration,
    })
}
