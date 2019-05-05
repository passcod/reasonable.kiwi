#[derive(Debug)]
pub enum E {
    NoListID,
    NoMatchingList,
    NoDescription,
    KeyTooShort,
    BadKeyFormat,
    CannotDecrypt,
    CannotUpdateUser,
    Twitter(egg_mode::error::Error),
    Encoding(base65536::Error),
    Database(diesel::result::Error),
}

impl From<egg_mode::error::Error> for E {
    fn from(err: egg_mode::error::Error) -> Self {
        E::Twitter(err)
    }
}

impl From<base65536::Error> for E {
    fn from(err: base65536::Error) -> Self {
        E::Encoding(err)
    }
}

impl From<diesel::result::Error> for E {
    fn from(err: diesel::result::Error) -> Self {
        E::Database(err)
    }
}
