<h1>Ocian-Rust!</h1>

> <p>Example of backend for study using the company Ocian - Barra Bonita, as a brand.</p>

<p> Backend 100% made in Rust 🦀 with Actix-Web, using less than 5 MB of RAM and high speed! 🚀 </p>
<p>This project contains 3 contact forms stored in the Postgresql database and another for resumes sent via email.</p>
<br/>

<h2>Build</h2>
<p>To compile the project, download the repository and run the following command:</p>

    $ cargo run --release

<br/>
<p>Create the .env file in the project root with the following example information:</p>

    DATABASE_HOST="yourserver.com"
    DATABASE_PORT="5432"
    DATABASE_NAME="your_database_name"
    DB_USER="your_database_user"
    DB_PASSWORD="your_database_password"
    
    HOST="smtp.gmail.com"
    PORT=587
    USERNAME="yourgmailfrom@gmail.com"
    PASSWORD="your_smtp_account_password"
    EMAIL="yourgmailto@gmail.com"


<p>Remembering that the information above is TOTALLY an example, so it is possible to use any other SMTP server, for example.</p>
<br/>

<h2>Endpoints</h2>

- <b>POST</b> http://0.0.0.0/send_proposal
<br/>

      -> Json
      {
        "name": "Claudio Oswaldo",
        "email": "claudiooswaldo@hotmail.com",
        "phone": "(14) 984205190",
        "subject": "I would like to make a proposal!",
        "message": "Proposal..."
      }

---

- <b>POST</b> http://0.0.0.0/send_contact_us
<br/>

      -> Json
      {
        "name": "Claudio Oswaldo",
        "email": "claudiooswaldo@hotmail.com",
        "phone": "(14) 984205190",
        "subject": "I would like to get in touch!",
        "message": "Contact..."
      }

---

- <b>POST</b> http://0.0.0.0/send_work_with_us
<br/>

      -> Multipart/form-data
      
        "name": "Claudio Oswaldo",
        "email": "claudiooswaldo@hotmail.com",
        "phone": "(14) 984205190",
        "file": yourfile.pdf/.docx/.word,
        "message": "Contact..."
      

---

- <b>POST</b> http://0.0.0.0/send_support
<br/>

      -> Json
      {
        "name": "Claudio Oswaldo",
        "email": "claudiooswaldo@hotmail.com",
        "phone": "(14) 984205190",
        "subject": "I need help from support!",
        "message": "I need help..."
      }
      

---

