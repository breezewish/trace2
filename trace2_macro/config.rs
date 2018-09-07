use syn;

#[derive(Debug)]
pub struct Config {
    pub ignore: bool,
}

impl Config {
    fn from_raw_pairs(pairs: Vec<RawConfigPair>) -> Config {
        let mut config = Config::default();
        for pair in pairs {
            match pair.name.to_string().as_str() {
                "ignore" => {
                    assert!(pair.value.is_none());
                    config.ignore = true;
                }
                _ => panic!("Unexpected configuration: {}", pair.name),
            }
        }
        config
    }
}

impl syn::synom::Synom for Config {
    fn parse(cursor: syn::buffer::Cursor) -> syn::synom::PResult<Self> {
        let result =
            syn::punctuated::Punctuated::<RawConfigPair, Token![,]>::parse_terminated(cursor);
        result.map(|(pairs, cursor)| (Config::from_raw_pairs(pairs.into_iter().collect()), cursor))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config { ignore: false }
    }
}

#[derive(Debug)]
struct RawConfigPair {
    name: syn::Ident,
    value: Option<syn::Lit>,
}

impl syn::synom::Synom for RawConfigPair {
    named!(parse -> Self, do_parse!(
        name: syn!(syn::Ident) >>
        value: option!(
            do_parse!(
                punct!(=) >>
                v: syn!(syn::Lit) >>
                (v)
            )
        ) >>
        (RawConfigPair {
            name,
            value,
        })
    ));
}

#[cfg(test)]
mod test {
    use super::{Config, RawConfigPair};

    use syn;

    #[test]
    fn parse_raw_config() {
        let pair = syn::parse_str::<RawConfigPair>("ignore").unwrap();
        assert_eq!(pair.name, "ignore");
        assert_eq!(pair.value, None);

        let pair = syn::parse_str::<RawConfigPair>("foo = \"bar\"").unwrap();
        assert_eq!(pair.name, "foo");
        let value = pair.value.unwrap();
        assert_eq!(quote!(#value).to_string(), "\"bar\"");

        let pair = syn::parse_str::<RawConfigPair>("boz = 123").unwrap();
        assert_eq!(pair.name, "boz");
        let value = pair.value.unwrap();
        assert_eq!(quote!(#value).to_string(), "123");
    }

    #[test]
    fn parse_config() {
        let config = syn::parse_str::<Config>("ignore").unwrap();
        assert_eq!(config.ignore, true);

        let config = syn::parse_str::<Config>("").unwrap();
        assert_eq!(config.ignore, false);
    }
}
