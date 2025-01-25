/// Macro for defining error enums with automatic Display, Error implementations, and utility methods
///
/// Generates an error enum with variants, along with:
/// - `Display` implementation using provided error messages
/// - `Error` trait implementation
/// - `description()` method returning static error messages
/// - `code()` method returning variant-specific numeric codes
/// - Optional defmt::Format derivation for test/debug configurations
///
/// # Arguments
/// * `name` - The name of the enum to create
/// * `variant => message` - Variants and their associated error messages
///
/// # Example
/// ```
/// define_peripheral_error_enum!(UartError,
///     Timeout => "Transaction timed out",
///     Framing => "Framing error detected",
///     Parity => "Invalid parity configuration"
/// );
/// ```
#[macro_export]
macro_rules! define_peripheral_error_enum {
    ($name:ident, $( $variant:ident => $message:expr ),* $(,)?) => {
        #[derive(Debug, PartialEq)]
        #[cfg_attr(any(test, feature = "debug"), derive(defmt::Format))]
        pub enum $name {
            $( $variant, )*
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $( $name::$variant => write!(f, $message), )*
                }
            }
        }

        impl $name {
            /// Returns the static error message associated with this variant
            pub fn description(&self) -> &'static str {
                match self {
                    $( $name::$variant => $message, )*
                }
            }

            /// Returns the numeric error code corresponding to this variant
            ///
            /// Codes are assigned based on the variant's ordinal position
            pub fn code(&self) -> u16 {
                match self {
                    $( $name::$variant => $name::$variant as u16, )*
                }
            }
        }

        impl core::error::Error for $name {}
    };
}

/// Macro for implementing error conversion with a default variant
///
/// Creates a From implementation that converts from one error type to another,
/// always mapping to a specified default variant of the target error type
///
/// # Arguments
/// * `from` - Source error type to convert from
/// * `to` - Target error type to convert into
/// * `default` - Default variant to use for conversion
///
/// # Example
/// ```
/// impl_error_conversion!(hal::spi::Error, AppError, { CommunicationFailure });
/// ```
#[macro_export]
macro_rules! impl_error_conversion {
    ($from:ty, $to:ty, { $default:ident }) => {
        impl From<$from> for $to {
            fn from(_error: $from) -> Self {
                Self::$default
            }
        }
    };
}