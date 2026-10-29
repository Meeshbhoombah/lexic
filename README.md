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
These instructions are written for MacOS. Lexic was built on Apple's Unix. 
Instructions for Windows and Linux have not been considered, contributions 
detailing how to start developing Lexic on Windows/Linux are welcome.

##### Homebrew
##### PostgreSQL
##### Rust
"A language empowering everyone to build reliable and efficient software."
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
The source code that contains this README  is the primary implementation of 
Lexic (written in Rust).

##### Cargo Watch
"Cargo Watch watches over your project's source for changes, and runs Cargo 
commands when they occur."

Navigate to your $ROOT directory. `cargo` `watch` needs to be installed 
as the SSOT development insturctions leverage `cargo` watch for live-reloading
the server as changes are made to the source code.
```
cd ~
```

Then,
```
cargo install cargo-watch
```
Lexic uses `cargo` `watch` to run the Node in development via `cargo` `run`.

##### Bun

#### Installation
1. Fork lexic. At the top of the page on GitHub, look for the "Fork" button. 
Click it to create your editable extension of Lexic's source code.

2. Clone your fork.
```
git clone <YOUR_FORK_URI>
```

3. Navigate into your cloned fork of Lexic.
```
cd lexic
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

###### File Structure
Use this `tree` command to view the entire file structure of Lexic.
```
tree -a -I '.git|node_modules|target|.next|.turbo|coverage|dist|build|.DS_Store'
```

### Contributing

