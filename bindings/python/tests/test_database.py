import sqlite3

import pytest

import limbo

test_database = "tests/database.db"
providers = ("provider", ["sqlite3", "limbo"])


@pytest.mark.parametrize(*providers)
def test_fetchall_select_all_users(provider):
    conn = connect(provider, test_database)
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM users")

    users = cursor.fetchall()
    assert users
    assert users == [(1, "alice"), (2, "bob")]


@pytest.mark.parametrize(*providers)
def test_fetchall_select_user_ids(provider):
    conn = connect(provider, test_database)
    cursor = conn.cursor()
    cursor.execute("SELECT id FROM users")

    user_ids = cursor.fetchall()
    assert user_ids
    assert user_ids == [(1,), (2,)]


@pytest.mark.parametrize(*providers)
def test_fetchone_select_all_users(provider):
    conn = connect(provider, test_database)
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM users")

    alice = cursor.fetchone()
    assert alice
    assert alice == (1, "alice")

    bob = cursor.fetchone()
    assert bob
    assert bob == (2, "bob")


@pytest.mark.parametrize(*providers)
def test_fetchone_select_max_user_id(provider):
    conn = connect(provider, test_database)
    cursor = conn.cursor()
    cursor.execute("SELECT MAX(id) FROM users")

    max_id = cursor.fetchone()
    assert max_id
    assert max_id == (2,)


@pytest.mark.parametrize(*providers)
def test_blob(provider):
    conn = connect(provider, test_database)
    cursor = conn.cursor()
    cursor.execute("SELECT data FROM blobs")

    blobs = cursor.fetchall()
    assert len(blobs) == 256
    for i, row in enumerate(blobs):
        expected = bytes.fromhex(f"{i:02x}")
        assert row[0] == expected
        assert type(row[0]) is bytes


def connect(provider, database):
    if provider == "limbo":
        return limbo.connect(database)
    if provider == "sqlite3":
        return sqlite3.connect(database)
    raise Exception(f"Provider `{provider}` is not supported")
