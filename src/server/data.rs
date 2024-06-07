use sqlite::{Connection, OpenFlags};
struct Database;

impl Database {
    pub fn new() -> Database {
        Database
    }

    pub fn open(&self) -> Result<Connection, String> {
        let mut dir = std::env::current_dir().unwrap_or_default();
        dir = dir.join("sqlite");
        if !&dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
        let database = dir.join("database.sqlite3");
        let flags = OpenFlags::new().with_read_only();

        Connection::open(database).map_err(|e| e.to_string())
    }

    pub fn create_address_table(&self) {}
}
