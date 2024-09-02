#!/usr/bin/env python3

import sqlite3

conn = sqlite3.connect("database.db")
cursor = conn.cursor()

# Create the user table
cursor.execute("""
    CREATE TABLE IF NOT EXISTS users (
        id INT PRIMARY KEY,
        username TEXT
    )
""")

users_list = [
    "alice",
    "bob",
]

for user in users_list:
    cursor.execute(
        """
            INSERT INTO users (name)
            VALUES (?)
        """,
        (user),
    )

conn.commit()
conn.close()
