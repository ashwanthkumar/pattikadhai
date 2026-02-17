use rusqlite::Connection;

fn run_migrations(conn: &Connection) {
    conn.execute_batch(include_str!("../migrations/001_initial.sql")).unwrap();
    conn.execute_batch(include_str!("../migrations/002_seed_genres.sql")).unwrap();
    conn.execute_batch(include_str!("../migrations/003_seed_stories.sql")).unwrap();
}

#[test]
fn migrations_create_all_tables() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn);

    // Check all tables exist
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(tables.contains(&"genres".to_string()));
    assert!(tables.contains(&"stories".to_string()));
    assert!(tables.contains(&"story_parts".to_string()));
    assert!(tables.contains(&"audio_jobs".to_string()));
}

#[test]
fn seed_data_has_expected_genre_count() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn);

    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM genres", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 6);
}

#[test]
fn seed_data_has_expected_sample_story_count() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn);

    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM stories WHERE is_sample = 1", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 4);
}

#[test]
fn unique_constraint_on_story_part_number() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn);

    conn.execute(
        "INSERT INTO stories (id, title, genre_id) VALUES ('test-story', 'Test', 'adventure')",
        [],
    ).unwrap();

    conn.execute(
        "INSERT INTO story_parts (id, story_id, part_number, content) VALUES ('p1', 'test-story', 1, 'Part 1')",
        [],
    ).unwrap();

    // Inserting same story_id + part_number should fail
    let result = conn.execute(
        "INSERT INTO story_parts (id, story_id, part_number, content) VALUES ('p2', 'test-story', 1, 'Duplicate')",
        [],
    );
    assert!(result.is_err());
}

#[test]
fn story_status_check_constraint() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn);

    // Valid status should work
    let result = conn.execute(
        "INSERT INTO stories (id, title, genre_id, status) VALUES ('s1', 'Test', 'adventure', 'draft')",
        [],
    );
    assert!(result.is_ok());

    // Invalid status should fail
    let result = conn.execute(
        "INSERT INTO stories (id, title, genre_id, status) VALUES ('s2', 'Test', 'adventure', 'invalid')",
        [],
    );
    assert!(result.is_err());
}

#[test]
fn cascade_delete_story_parts() {
    let conn = Connection::open_in_memory().unwrap();
    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    run_migrations(&conn);

    conn.execute(
        "INSERT INTO stories (id, title, genre_id) VALUES ('del-test', 'Delete Test', 'adventure')",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO story_parts (id, story_id, part_number, content) VALUES ('dp1', 'del-test', 1, 'Part 1')",
        [],
    ).unwrap();

    // Delete story should cascade to parts
    conn.execute("DELETE FROM stories WHERE id = 'del-test'", []).unwrap();

    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM story_parts WHERE story_id = 'del-test'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn migration_versions_are_sequential() {
    // Verify our migration numbering is sequential
    let migration_files = [
        "001_initial.sql",
        "002_seed_genres.sql",
        "003_seed_stories.sql",
    ];

    for (i, file) in migration_files.iter().enumerate() {
        let expected_prefix = format!("{:03}", i + 1);
        assert!(file.starts_with(&expected_prefix), "Migration {} should start with {}", file, expected_prefix);
    }
}
