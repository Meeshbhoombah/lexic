# Lexic
Decentralized Messaging

## TODO
- [ ] Fix server failing silently if:
    + PostgreSQL is not running

## Getting Started
<!--
lexic is available at:
- [lexic.someoperatorwebsite.com](#)
- [lexic.someotheroperatorwebsite.com](#)
-->

### Running a Node
#### Prerequisites
#### Installation
#### Deployment

### Development
#### Prequisites
##### MacOS
These instructions are written for MacOS. Lexic was built on Apple's Unix 
system. Instructions for Windows and Linux have not been considered, 
contributions detailing how to start developing Lexic on Windows/Linux
are welcome.

##### Homebrew
##### PostgreSQL
##### Bun

#### Installation
1. Fork lexic.
```
```

2. Clone your fork.
```
```

3. Navigate into your cloned fork of lexic.
```
```

4. Build Lexic.
```
cargo build
```

5. Navigate into the browser client's directory.
```
cd browser
```

6. Install the browser client's dependencies.
```
bun install
```

7. Create a development database. First, activate the `psql` console.
```
psql -d postgres
```

8. Then, at the PostgreSQL prompt, execute:
```
CREATE ROLE lexic WITH LOGIN PASSWORD 'local-development-only';
CREATE DATABASE lexic OWNER lexic;
```

9. Exit the prompt:
```
\q
```

10. Duplicate `.env.example` as your local `.env` file.
```
cp .env.example .env
```

11. Navigate into the browser client's directory.
```
cd browser
```

12. Duplicate `.env.local.example` as your local `.env` file.
```
cp .env.local.example .env.local
```

#### Building
Start a local development instance of Postgres.
```
brew services start postgresql
```

Start the Lexic node.
```
cargo watch -x run
```

Navigate into the browser client's directory.
```
cd browser
```

Start the browser client development server.
```
bun --bun run dev
```

### Contributing

