CREATE TABLE support (
    id SERIAL,
    name VARCHAR(50) NOT NULL,
    email VARCHAR(50) NOT NULL,
    phone VARCHAR(20) NOT NULL,
    subject VARCHAR(50) NOT NULL,
    message VARCHAR(500) NOT NULL
);
