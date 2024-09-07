#!/usr/bin/env python3

import os
import sqlite3

from faker import Faker

dirname = os.path.dirname(__file__)
db_path = os.path.join(dirname, "database.db")

if os.path.exists(db_path):
    os.remove(db_path)
conn = sqlite3.connect(db_path)
cursor = conn.cursor()

fake = Faker()


#### USERS TABLE ####

cursor.execute("""
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        username TEXT
    );
""")

users_list = [
    "alice",
    "bob",
]

for user in users_list:
    cursor.execute(
        """
        INSERT INTO users (username)
        VALUES (?)
        """,
        (user,),
    )


# #### BLOBS TABLE ####

cursor.execute("""
    CREATE TABLE IF NOT EXISTS blobs (
        id INTEGER PRIMARY KEY,
        data BLOB
    )
""")

Faker.seed(0)
for i in range(256):
    data = bytes.fromhex(f"{i:02x}")
    cursor.execute(
        """
        INSERT INTO blobs (data)
        VALUES (?)
        """,
        (data,),
    )

conn.commit()
conn.close()
