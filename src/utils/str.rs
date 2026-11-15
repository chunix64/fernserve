use core::fmt::Write;

use defmt::{Debug2Format, error};
use heapless::string::String;

use crate::configs::ERROR_STRING_MAX_LENGTH;

fn try_write<const N: usize>(
    buffer: &mut String<N>,
    args: core::fmt::Arguments<'_>,
) -> Result<(), core::fmt::Error> {
    buffer.clear();
    buffer.write_fmt(args)
}

pub fn error_to_string(error: anyhow::Error) -> String<ERROR_STRING_MAX_LENGTH> {
    let mut buffer = String::<ERROR_STRING_MAX_LENGTH>::new();

    let result = try_write(&mut buffer, format_args!("{:#}", error))
        .or_else(|_| try_write(&mut buffer, format_args!("{}", error)))
        .or_else(|_| try_write(&mut buffer, format_args!("{:#?}", error)))
        .or_else(|_| try_write(&mut buffer, format_args!("{:?}", error)));

    if let Err(parsing_error) = result {
        error!(
            "[UTILS] error_to_string failed: {:#?} for {:#?}",
            parsing_error,
            Debug2Format(&error)
        );
    }

    buffer
}

pub fn usize_to_string(number: usize) -> String<20> {
    let mut buffer = String::<20>::new();

    if let Err(parsing_error) = write!(buffer, "{}", number) {
        error!(
            "[UTILS] usize_to_string failed: {:#?} for {}",
            parsing_error, number
        );
    }

    buffer
}

pub fn bool_to_str(flag: bool) -> &'static str {
    if flag { "true" } else { "false" }
}
