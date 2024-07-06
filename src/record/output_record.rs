use crate::util::pretty_printed_json;


#[derive(Clone, Debug)]
pub struct OutputRecord {
    pub key: String,
    pub value: String,
}

impl OutputRecord {
    pub fn key(&self) -> &String {
        &self.key
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn print(&self) {
        let key = &self.key;
        let value = &self.value;

        match pretty_printed_json(value) {
            Some(pretty) => {
                println!("\noutput {key:#?} \n{pretty}\n");
            }
            None => {
                println!("\noutput {key:#?} \n{value:?}\n");
            }
        }
    }
}
