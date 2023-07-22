use std::fmt::Display;

use serde::Serialize;

use crate::cli::OutputFormat;

#[derive(Serialize)]
pub struct TableCellOption<T>(Option<T>);

impl<T> From<Option<T>> for TableCellOption<T> {
    fn from(value: Option<T>) -> Self {
        TableCellOption(value)
    }
}

impl<T: Display> Display for TableCellOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = &self.0 {
            value.fmt(f)
        } else {
            f.write_str("-")
        }
    }
}

#[derive(Serialize)]
pub struct TableCellSlice<'a, T>(&'a [T]);

impl<'a, T> From<&'a [T]> for TableCellSlice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        TableCellSlice(value)
    }
}

impl<'a, T: Display> Display for TableCellSlice<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.0.len();

        if len > 0 {
            self.0[0].fmt(f)?;
        }

        for i in 1..len {
            f.write_str(",")?;
            self.0[i].fmt(f)?;
        }

        Ok(())
    }
}

pub trait TableOutputRow {
    fn print_name(&self);
    fn print_header();
    fn print_row(&self);
}

pub trait TableOutputDisplay {
    fn print_names(self);
    fn print_table(self);
    fn print_table_with_headers(self);
}

impl<I: IntoIterator<Item = T>, T: TableOutputRow> TableOutputDisplay for I {
    fn print_names(self) {
        for row in self {
            row.print_name();
        }
    }

    fn print_table(self) {
        for row in self {
            row.print_row();
        }
    }

    fn print_table_with_headers(self) {
        T::print_header();
        self.print_table();
    }
}

pub trait SerializableOutputDisplay {
    fn print_json(&self) -> Result<(), serde_json::Error>;
    fn print_json_pretty(&self) -> Result<(), serde_json::Error>;
    fn print_yaml(&self) -> Result<(), serde_yaml::Error>;
}

impl<T: ?Sized + Serialize> SerializableOutputDisplay for T {
    fn print_json(&self) -> Result<(), serde_json::Error> {
        let output = serde_json::to_string(self)?;
        print!("{output}");

        Ok(())
    }

    fn print_json_pretty(&self) -> Result<(), serde_json::Error> {
        let output = serde_json::to_string_pretty(self)?;
        print!("{output}");

        Ok(())
    }

    fn print_yaml(&self) -> Result<(), serde_yaml::Error> {
        let output = serde_yaml::to_string(self)?;
        print!("{output}");

        Ok(())
    }
}

pub trait CliPrint {
    fn print(self, format: OutputFormat) -> anyhow::Result<()>;
}

impl<T: Serialize + TableOutputDisplay> CliPrint for T {
    fn print(self, format: OutputFormat) -> anyhow::Result<()> {
        match format {
            OutputFormat::Names => self.print_names(),
            OutputFormat::Table => self.print_table(),
            OutputFormat::TableWithHeaders => self.print_table_with_headers(),
            OutputFormat::Json => self.print_json()?,
            OutputFormat::JsonPretty => self.print_json_pretty()?,
            OutputFormat::Yaml => self.print_yaml()?,
        }

        Ok(())
    }
}
