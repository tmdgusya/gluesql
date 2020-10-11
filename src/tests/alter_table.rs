use crate::*;
use Value::*;

fn run(mut tester: impl tests::Tester, test_cases: &[(&str, Result<Payload>)]) {
    test_cases.iter().for_each(|(sql, expected)| {
        let found = tester.run(sql);

        assert_eq!(expected, &found);
    });
}

pub fn rename(tester: impl tests::Tester) {
    let test_cases = [
        ("CREATE TABLE Foo (id INTEGER);", Ok(Payload::Create)),
        (
            "INSERT INTO Foo VALUES (1), (2), (3);",
            Ok(Payload::Insert(3)),
        ),
        ("SELECT id FROM Foo", Ok(select!(I64; 1; 2; 3))),
        ("ALTER TABLE Foo RENAME TO Bar;", Ok(Payload::AlterTable)),
        ("SELECT id FROM Bar", Ok(select!(I64; 1; 2; 3))),
        (
            "ALTER TABLE Bar RENAME COLUMN id TO new_id",
            Ok(Payload::AlterTable),
        ),
        ("SELECT new_id FROM Bar", Ok(select!(I64; 1; 2; 3))),
        ("SELECT new_id FROM Bar", Ok(select!(I64; 1; 2; 3))),
        (
            "ALTER TABLE Bar RENAME COLUMN hello TO idid",
            Err(AlterTableError::ColumnNotFound.into()),
        ),
    ];

    run(tester, &test_cases);
}

pub fn add_drop(tester: impl tests::Tester) {
    let test_cases = [
        ("CREATE TABLE Foo (id INTEGER);", Ok(Payload::Create)),
        ("INSERT INTO Foo VALUES (1), (2);", Ok(Payload::Insert(2))),
        ("SELECT * FROM Foo;", Ok(select!(I64; 1; 2))),
        (
            "ALTER TABLE Foo ADD COLUMN amount INTEGER",
            Err(AlterTableError::DefaultValueRequired("amount INT".to_owned()).into()),
        ),
        (
            "ALTER TABLE Foo ADD COLUMN id INTEGER",
            Err(AlterTableError::ColumnAlreadyExists("id".to_owned()).into()),
        ),
        (
            "ALTER TABLE Foo ADD COLUMN amount INTEGER DEFAULT 10",
            Ok(Payload::AlterTable),
        ),
        ("SELECT * FROM Foo;", Ok(select!(I64 I64; 1 10; 2 10))),
        (
            "ALTER TABLE Foo ADD COLUMN opt BOOLEAN NULL",
            Ok(Payload::AlterTable),
        ),
        (
            "SELECT * FROM Foo;",
            Ok(select!(
                I64 I64 OptBool;
                1 10 None;
                2 10 None
            )),
        ),
        (
            "ALTER TABLE Foo ADD COLUMN opt2 BOOLEAN NULL DEFAULT true",
            Ok(Payload::AlterTable),
        ),
        (
            "SELECT * FROM Foo;",
            Ok(select!(
                I64 I64 OptBool OptBool;
                1   10  None    Some(true);
                2   10  None    Some(true)
            )),
        ),
        (
            "ALTER TABLE Foo ADD COLUMN something INTEGER DEFAULT (SELECT id FROM Bar LIMIT 1)",
            Err(ValueError::ExprNotSupported("(SELECT id FROM Bar LIMIT 1)".to_owned()).into()),
        ),
    ];

    run(tester, &test_cases);
}
