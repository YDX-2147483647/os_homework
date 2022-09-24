#[derive(Debug, PartialEq)]
pub enum OperatorRole {
    Reader,
    Writer,
}

#[derive(Debug, PartialEq)]
pub struct Operator {
    /// 序号
    pub id: u32,
    /// 角色
    pub role: OperatorRole,
    /// 操作开始时刻，单位为秒，正数
    pub start_at: f32,
    /// 操作持续时间，正数
    pub duration: f32,
}

#[derive(Debug)]
pub enum OperatorParseError {
    InvalidId,
    InvalidRole,
    InvalidStartAt,
    InvalidDuration,
    TooManyFields,
}

impl Operator {
    /// Parse an operator from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use ex_2::*;
    ///
    /// assert_eq!(
    ///     Operator::from("1 R 3.3 5").unwrap(),
    ///     Operator {
    ///         id: 1,
    ///         role: OperatorRole::Reader,
    ///         start_at: 3.3,
    ///         duration: 5.,
    ///     }
    /// );
    /// ```
    pub fn from(line: &str) -> Result<Operator, OperatorParseError> {
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
}
