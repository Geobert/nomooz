use crate::xkb_parser::XkbParser;

pub struct Labels {
    pub all: Vec<Vec<String>>,
    pub column: Vec<Vec<String>>,
}

// Returns labels to be displayed on grid
impl Labels {
    pub fn rebuild(xkb_parser: &XkbParser) -> Self {
        // Main 30 alpha keys
        let all_keys: Vec<u32> = (16..26).chain(30..40).chain(44..54).collect();
        // Home row keys
        let hr_keys: Vec<u32> = (30..40).collect();

        let all: Vec<Vec<String>> = all_keys
            .iter()
            .map(|&line| {
                hr_keys
                    .iter()
                    .map(|&col| {
                        format!(
                            "{}   {}",
                            xkb_parser.key_code_to_char(col),
                            xkb_parser.key_code_to_char(line)
                        )
                    })
                    .collect()
            })
            .collect();

        let column: Vec<Vec<String>> = all_keys
            .iter()
            .map(|&key| vec![xkb_parser.key_code_to_char(key).to_string()])
            .collect();

        Labels {
            all,
            column,
        }
    }
}
