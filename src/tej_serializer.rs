use std::io::Write;
pub trait MeasurementCollector {
    type Error;
    type Data;
    fn start(&mut self) -> Result<(), Self::Error>;
    fn end(&mut self) -> Result<Self::Data, Self::Error>;
    fn timestamp(&mut self, value: String) -> Result<(), Self::Error>;
    fn measurement(&mut self, name: &str, value: f64) -> Result<(), Self::Error>;
    fn start_group(&mut self, group: &str) -> Result<(), Self::Error>;
    fn end_group(&mut self) -> Result<(), Self::Error>;
}
/// Serialize a series of measurements into a ThinEdgeJson byte-string.
/// Perform no check beyond the fact that groups are properly closed.
pub struct ThinEdgeJsonSerializer {
    buffer: Vec<u8>,
    is_within_group: bool,
    needs_separator: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum ThinEdgeJsonSerializationError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    MeasurementCollectorError(#[from] MeasurementStreamError),
}

#[derive(thiserror::Error, Debug)]
pub enum MeasurementStreamError {
    #[error("Unexpected time stamp within a group")]
    UnexpectedTimestamp,

    #[error("Unexpected end of data")]
    UnexpectedEndOfData,

    #[error("Unexpected end of group")]
    UnexpectedEndOfGroup,

    #[error("Unexpected start of group")]
    UnexpectedStartOfGroup,
}

impl ThinEdgeJsonSerializer {
    pub fn new() -> Self {
        ThinEdgeJsonSerializer {
            buffer: Vec::new(),
            is_within_group: false,
            needs_separator: false,
        }
    }
}

impl MeasurementCollector for ThinEdgeJsonSerializer {
    type Error = ThinEdgeJsonSerializationError;
    type Data = Vec<u8>;
    fn start(&mut self) -> Result<(), Self::Error> {
        self.buffer.push(b'{');
        self.needs_separator = false;
        Ok(())
    }

    fn end(&mut self) -> Result<Self::Data, Self::Error> {
        if self.is_within_group {
            return Err(MeasurementStreamError::UnexpectedEndOfData.into());
        }

        self.buffer.push(b'}');
        Ok(self.buffer)
    }

    fn timestamp(&mut self, value: String) -> Result<(), Self::Error> {
        if self.is_within_group {
            return Err(MeasurementStreamError::UnexpectedTimestamp.into());
        }

        if self.needs_separator {
            self.buffer.push(b',');
        }
        self.buffer
            .write_fmt(format_args!("\"time\":\"{}\"", value.to_rfc3339()))?;
        self.needs_separator = true;
        Ok(())
    }

    fn measurement(&mut self, name: &str, value: f64) -> Result<(), Self::Error> {
        if self.needs_separator {
            self.buffer.push(b',');
        }
        self.buffer
            .write_fmt(format_args!("\"{}\":{}", name, value))?;
        self.needs_separator = true;
        Ok(())
    }

    fn start_group(&mut self, group: &str) -> Result<(), Self::Error> {
        if self.is_within_group {
            return Err(MeasurementStreamError::UnexpectedStartOfGroup.into());
        }

        if self.needs_separator {
            self.buffer.push(b',');
        }
        self.buffer.write_fmt(format_args!("\"{}\":{{", group))?;
        self.needs_separator = false;
        Ok(())
    }

    fn end_group(&mut self) -> Result<(), Self::Error> {
        if !self.is_within_group {
            return Err(MeasurementStreamError::UnexpectedEndOfGroup.into());
        }

        self.buffer.push(b'}');
        self.needs_separator = true;
        Ok(())
    }
}