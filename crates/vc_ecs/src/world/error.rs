use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetEntityMutByIdError {
    InfoNotFound,
    ComponentIsImmutable,
    ComponentNotFound,
}

impl fmt::Display for GetEntityMutByIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetEntityMutByIdError::InfoNotFound => {
                f.write_str("the `ComponentInfo` could not be found")
            }
            GetEntityMutByIdError::ComponentIsImmutable => {
                f.write_str("the `Component` is immutable")
            }
            GetEntityMutByIdError::ComponentNotFound => {
                f.write_str("the `Component` could not be found")
            }
        }
    }
}

impl core::error::Error for GetEntityMutByIdError {}
