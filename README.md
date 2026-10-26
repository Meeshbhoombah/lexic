# Lexic
Decentralized Messaging

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
##### Homebrew
##### PostgreSQL
##### Bun

#### Installation
Fork lexic.
```
```

Clone your fork.
```
```

Navigate into your cloned fork of lexic.
```
```

Create a development database. First, activate the `psql` console.
```
psql -d postgres
```

Then, at the PostgreSQL prompt, execute:
```
CREATE ROLE lexic WITH LOGIN PASSWORD 'local-development-only';
CREATE DATABASE lexic OWNER lexic;
```

Exit the prompt:
```
\q
```

Duplicate `.env.example` as your local `.env` file.
```
cp .env.example .env
```

#### Building
Start a local development instance of Postgres.
```
brew services start postgresql
```

### Contributing

