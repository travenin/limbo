#!/usr/bin/env python3

import os
import sqlite3
from faker import Faker

db_path = "testing.db"
if os.path.exists(db_path):
    os.remove(db_path)
conn = sqlite3.connect(db_path)
cursor = conn.cursor()

fake = Faker()


#### USERS TABLE ####

cursor.execute("""
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        first_name TEXT,
        last_name TEXT,
        email TEXT,
        phone_number TEXT,
        address TEXT,
        city TEXT,
        state TEXT,
        zipcode TEXT,
        age INTEGER
    )
""")

Faker.seed(0)
for _ in range(10000):
    first_name = fake.first_name()
    last_name = fake.last_name()
    email = fake.email()
    phone_number = fake.phone_number()
    address = fake.street_address()
    city = fake.city()
    state = fake.state_abbr()
    zipcode = fake.zipcode()
    age = fake.random_int(min=1, max=100)

    cursor.execute(
        """
        INSERT INTO users (first_name, last_name, email, phone_number, address, city, state, zipcode, age)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            first_name,
            last_name,
            email,
            phone_number,
            address,
            city,
            state,
            zipcode,
            age,
        ),
    )


#### PRODUCTS TABLE ####

cursor.execute("""
    CREATE TABLE IF NOT EXISTS products (
        id INTEGER PRIMARY KEY,
        name TEXT,
        price REAL
    )
""")

product_list = [
    "hat",
    "cap",
    "shirt",
    "sweater",
    "sweatshirt",
    "shorts",
    "jeans",
    "sneakers",
    "boots",
    "coat",
    "accessories",
]

Faker.seed(0)
for product in product_list:
    price = fake.random_int(min=1, max=100)
    cursor.execute(
        """
        INSERT INTO products (name, price)
        VALUES (?, ?)
        """,
        (product, price),
    )


#### BLOBS TABLE ####

cursor.execute("""
    CREATE TABLE IF NOT EXISTS blobs (
        id INTEGER PRIMARY KEY,
        text_data BLOB,
        random_data BLOB
    )
""")

Faker.seed(0)
for _ in range(100):
    text_data = fake.text(50).encode()
    random_data = fake.binary(length=128)
    cursor.execute(
        """
        INSERT INTO blobs (text_data, random_data)
        VALUES (?, ?)
        """,
        (text_data, random_data),
    )

conn.commit()
conn.close()
