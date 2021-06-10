use ipnet::Ipv4Net;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize_ipv4net<S>(ipv4net: &Ipv4Net, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ipv4net.to_string().serialize(serializer)
}

pub fn deserialize_ipv4net<'de, D>(deserializer: D) -> Result<Ipv4Net, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|x| x.parse::<Ipv4Net>().map_err(|e| D::Error::custom(e)))
}

// Takes in a table of strings (vec of rows, each row is a vec of strings)
// Returns a vec of print lines
pub fn cli_table(table: Vec<Vec<&str>>) -> Vec<String> {
    let mut column_widths: Vec<usize> = Vec::new();

    for row in table.iter() {
        for (i, cell) in row.iter().enumerate() {
            let width = cell.len();
            if i >= column_widths.len() {
                column_widths.push(width);
            } else if column_widths[i] < width {
                column_widths[i] = width;
            }
        }
    }

    let mut lines: Vec<String> = Vec::new();

    for row in table.iter() {
        let mut line = String::new();

        for (i, cell) in row.iter().enumerate() {
            let max_width = column_widths[i];    
            let padding = max_width - cell.len();
            line.push_str(&" ".repeat(padding));
            line.push_str(cell);
            line.push(' '); // for spacing
        }
        line.pop(); // remove the last extra space

        lines.push(line);
    }

    lines
}