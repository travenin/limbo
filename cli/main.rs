mod opcodes_dictionary;

use clap::{Parser, ValueEnum};
use cli_table::{Cell, Table};
use limbo_core::{Database, RowResult, Value};
use opcodes_dictionary::OPCODE_DESCRIPTIONS;
use rustyline::{error::ReadlineError, DefaultEditor};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum OutputMode {
    Raw,
    Pretty,
}

impl std::fmt::Display for OutputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Opts {
    database: PathBuf,
    sql: Option<String>,
    #[clap(short, long, default_value_t = OutputMode::Raw)]
    output_mode: OutputMode,
}

#[allow(clippy::arc_with_non_send_sync)]
fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts = Opts::parse();
    let path = opts.database.to_str().unwrap();
    let io = Arc::new(limbo_core::PlatformIO::new()?);
    let db = Database::open_file(io.clone(), path)?;
    let conn = db.connect();

    let interrupt_count = Arc::new(AtomicUsize::new(0));

    {
        let interrupt_count = Arc::clone(&interrupt_count);

        ctrlc::set_handler(move || {
            // Increment the interrupt count on Ctrl-C
            interrupt_count.fetch_add(1, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }

    if let Some(sql) = opts.sql {
        if sql.trim().starts_with('.') {
            handle_dot_command(io.clone(), &conn, &sql)?;
        } else {
            query(io.clone(), &conn, &sql, &opts.output_mode, &interrupt_count)?;
        }
        return Ok(());
    }
    let mut rl = DefaultEditor::new()?;
    let home = dirs::home_dir().unwrap();
    let history_file = home.join(".limbo_history");
    if history_file.exists() {
        rl.load_history(history_file.as_path())?;
    }
    println!("Limbo v{}", env!("CARGO_PKG_VERSION"));
    println!("Enter \".help\" for usage hints.");
    loop {
        let readline = rl.readline("limbo> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.to_owned())?;
                interrupt_count.store(0, Ordering::SeqCst);
                if line.trim().starts_with('.') {
                    handle_dot_command(io.clone(), &conn, &line)?;
                } else {
                    query(
                        io.clone(),
                        &conn,
                        &line,
                        &opts.output_mode,
                        &interrupt_count,
                    )?;
                }
            }
            Err(ReadlineError::Interrupted) => {
                // At prompt, increment interrupt count
                if interrupt_count.fetch_add(1, Ordering::SeqCst) >= 1 {
                    eprintln!("Interrupted. Exiting...");
                    break;
                }
                println!("Use .quit to exit or press Ctrl-C again to force quit.");
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                anyhow::bail!(err)
            }
        }
    }
    rl.save_history(history_file.as_path())?;
    Ok(())
}

fn display_help_message() {
    let help_message = r#"
Limbo SQL Shell Help
==============

Welcome to the Limbo SQL Shell! You can execute any standard SQL command here.
In addition to standard SQL commands, the following special commands are available:

Special Commands:
-----------------
.quit                      Stop interpreting input stream and exit.
.schema <table_name>       Show the schema of the specified table.
.opcodes                   Display all the opcodes defined by the virtual machine
.help                      Display this help message.

Usage Examples:
---------------
1. To quit the Limbo SQL Shell:
   .quit

2. To view the schema of a table named 'employees':
   .schema employees

3. To list all available SQL opcodes:
   .opcodes

Note:
-----
- All SQL commands must end with a semicolon (;).
- Special commands do not require a semicolon.

"#;

    println!("{}", help_message);
}

fn handle_dot_command(
    io: Arc<dyn limbo_core::IO>,
    conn: &limbo_core::Connection,
    line: &str,
) -> anyhow::Result<()> {
    let args: Vec<&str> = line.split_whitespace().collect();

    if args.is_empty() {
        return Ok(());
    }

    match args[0] {
        ".quit" => {
            println!("Exiting Limbo SQL Shell.");
            std::process::exit(0)
        }
        ".schema" => {
            let table_name = args.get(1).copied();
            display_schema(io, conn, table_name)?;
        }
        ".opcodes" => {
            if args.len() > 1 {
                for op in &OPCODE_DESCRIPTIONS {
                    if op.name.eq_ignore_ascii_case(args.get(1).unwrap()) {
                        println!("{}", op);
                    }
                }
            } else {
                for op in &OPCODE_DESCRIPTIONS {
                    println!("{}\n", op);
                }
            }
        }
        ".help" => {
            display_help_message();
        }
        _ => {
            println!("Unknown command: {}", args[0]);
            println!("Available commands:");
            println!("  .schema <table_name> - Display the schema for a specific table");
            println!(
                "  .opcodes             - Display all the opcodes defined by the virtual machine"
            );
        }
    }

    Ok(())
}

fn display_schema(
    io: Arc<dyn limbo_core::IO>,
    conn: &limbo_core::Connection,
    table: Option<&str>,
) -> anyhow::Result<()> {
    let sql = match table {
        Some(table_name) => format!(
            "SELECT sql FROM sqlite_schema WHERE type='table' AND name = '{}' AND name NOT LIKE 'sqlite_%'",
            table_name
        ),
        None => String::from(
            "SELECT sql FROM sqlite_schema WHERE type IN ('table', 'index') AND name NOT LIKE 'sqlite_%'"
        ),
    };

    match conn.query(sql) {
        Ok(Some(ref mut rows)) => {
            let mut found = false;
            loop {
                match rows.next_row()? {
                    RowResult::Row(row) => {
                        if let Some(Value::Text(schema)) = row.values.first() {
                            println!("{};", schema);
                            found = true;
                        }
                    }
                    RowResult::IO => {
                        io.run_once()?;
                    }
                    RowResult::Done => break,
                }
            }
            if !found {
                if let Some(table_name) = table {
                    println!("Error: Table '{}' not found.", table_name);
                } else {
                    println!("No tables or indexes found in the database.");
                }
            }
        }
        Ok(None) => {
            println!("No results returned from the query.");
        }
        Err(err) => {
            if err.to_string().contains("no such table: sqlite_schema") {
                return Err(anyhow::anyhow!("Unable to access database schema. The database may be using an older SQLite version or may not be properly initialized."));
            } else {
                return Err(anyhow::anyhow!("Error querying schema: {}", err));
            }
        }
    }

    Ok(())
}

fn query(
    io: Arc<dyn limbo_core::IO>,
    conn: &limbo_core::Connection,
    sql: &str,
    output_mode: &OutputMode,
    interrupt_count: &Arc<AtomicUsize>,
) -> anyhow::Result<()> {
    match conn.query(sql) {
        Ok(Some(ref mut rows)) => match output_mode {
            OutputMode::Raw => loop {
                if interrupt_count.load(Ordering::SeqCst) > 0 {
                    println!("Query interrupted.");
                    return Ok(());
                }

                match rows.next_row()? {
                    RowResult::Row(row) => {
                        for (i, value) in row.values.iter().enumerate() {
                            if i > 0 {
                                print!("|");
                            }
                            match value {
                                Value::Null => print!(""),
                                Value::Integer(i) => print!("{}", i),
                                Value::Float(f) => print!("{:?}", f),
                                Value::Text(s) => print!("{}", s),
                                Value::Blob(b) => {
                                    print!("{}", String::from_utf8_lossy(b))
                                }
                            }
                        }
                        println!();
                    }
                    RowResult::IO => {
                        io.run_once()?;
                    }
                    RowResult::Done => {
                        break;
                    }
                }
            },
            OutputMode::Pretty => {
                if interrupt_count.load(Ordering::SeqCst) > 0 {
                    println!("Query interrupted.");
                    return Ok(());
                }
                let mut table_rows: Vec<Vec<_>> = vec![];
                loop {
                    match rows.next_row()? {
                        RowResult::Row(row) => {
                            table_rows.push(
                                row.values
                                    .iter()
                                    .map(|value| match value {
                                        Value::Null => "".cell(),
                                        Value::Integer(i) => i.to_string().cell(),
                                        Value::Float(f) => f.to_string().cell(),
                                        Value::Text(s) => s.cell(),
                                        Value::Blob(b) => {
                                            format!("{}", String::from_utf8(b.to_vec()).unwrap())
                                                .cell()
                                        }
                                    })
                                    .collect(),
                            );
                        }
                        RowResult::IO => {
                            io.run_once()?;
                        }
                        RowResult::Done => break,
                    }
                }
                let table = table_rows.table();
                cli_table::print_stdout(table).unwrap();
            }
        },
        Ok(None) => {}
        Err(err) => {
            eprintln!("{}", err);
        }
    }
    // for now let's cache flush always
    conn.cacheflush()?;
    Ok(())
}
