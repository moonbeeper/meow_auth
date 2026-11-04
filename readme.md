# Meow auth2

> [!NOTE]
> wok IN progress >:3

hi, this is my little authentcation server. you know how it goes, unauthenticated user goes in, authenticated user goes out. That's it. That's the whole backend ted talk.

What actually works right now? An awful lot of things (thank god):

- **Frontend**. yes that's it. it has neon color which are very very cool and smushy buttons
- login + signup via _email otp codes_
- **TOTP** (2fa) two-factor authentication with recovery codes (the secrets to verify and generate them are stored encrypted, while the actual codes are derived from the secrets and not stored anywhere)
- **Passkeys** (webauthn). its the primary reason i removed the password login method, they are just easy to use and more secure ig
- **Sudo mode**, yes that thing from github that makes you re-authenticate for sensitive actions (or spooky scary actions). it works with all the login methods
- **Audit log** of basically everything that you do with even change diffs!
- **Oauth 2.1** and **OIDC providers** (OpenId) that work great this time (Has the typical oauth things like PKCE 256 only, auth code for getting the access token, no implicit stupid nonsense AND no stoopid refresh tokens)
- **Admin panel** that allows you to manage users, view their logs and manage their authorized and created apps
- **Email sending** nonsensical stuff like email verification and etc. Its just email notifications, typical stuff
- OpenApi docs with the Scalar Api explorer in the backend at "/scalar". Useful while looking at the api and testing it out

## what the hell is each folder for?

ah, young.. cat, that's a good great question that my past self didn't understand too. This is basically a (badly made) rust http backend
with its config, its database migration files, a proc macro subcrate for the boring repetitive stuff and a frontend that talks to it.

- [src/](src/) is teh whole server. yes, all of it. with even the mistakes included
- [settings/](settings/) is where the config files live. Although, they can be loaded from also the environment variables
- [migrations/](migrations/) is the blue elephant db (Postgresql) migration files. They are necessary to literally make the database work
- [frontend/](frontend/) is the SvelteKit app (wow)
- [macros/](macros/) is the proc macro crate that is used to generate useful stuff for the OpenApi docs and for the settings docs.

The insides of the backend are self explanatory, I did not make alien names like "atht" = "authentication" or "authz" = "authorization", that's stupid and confusing. Its pretty easy to follow the code and understand the spaggety underneath.

## settings

Settings are loaded from the [settings/](settings/) folder, environment variables, the root folder or a combination of those.

The settings files are in TOML format with its file name as "<environment>.toml". The "environment" its literally where it will be running. For example:

- `development.toml` is for development, `production.toml` is for production.

The backend will load by default the `development` environment, so that is `development.toml`. You can control what environment it will load by setting the `WORK_ENV` environment variable.

```bash
WORK_ENV=development cargo run
```

and if the file does not exist yet (or you pulled new keys and want them added to your toml), pass `-e <environment>`:

```bash
cargo run -- -e production # creates/updates settings/production.toml from the defaults
WORK_ENV=production cargo run # this is the one that actually loads production.toml
```

> [!WARNING]
> `WORK_ENV` picks what `settings/<environment>.toml` (or `<environment>.toml` in the root) gets loaded. startup does not create or edit settings files on its own, and a missing required value just makes it fail to parse and die.
> `-e <environment>` writes `settings/<environment>.toml` from the defaults in [src/settings.rs](src/settings.rs), but it does NOT switch the environment of the run you just started (that's still `WORK_ENV`, default `development`).

**You should take a look at the [settings/development.toml](settings/development.toml) file to see what values you can change and its explanation.**

## database

A Postgres database required to do... database things. The database connection is configured in the settings file but you'll need to manually migrate all the migrations files to create the database tables. You can do that by:

1. Installing the sqlx-cli tool via cargo:

```bash
cargo install sqlx-cli # This installs the management all supported databases that sqlx supports. You may tone it down a bit.
```

2. Making sure the .env file is set up with the correct database connection string. For example:

```bash
DATABASE_URL=postgres://meow:meow_pwd@localhost:5432/meow2_development
```

3. Running the migration:

```bash
cargo sqlx migrate run
# or
sqlx migrate run
```

You can run the development database by using docker compose:

```bash
docker compose up -d # The "-d" flag means "detached", which means it will run in the background. You can omit it if you want to see the logs of the services.
```

The database will be living at `localhost:5432` with the username `meow`, password `meow_pwd` and database name `meow2_development`.

The [.sqlx](.sqlx) folder is where the sqlx-cli tool will store its "offline query metadata". Its basically a cache of the queries that let you build the backend without having to connect to the database. Tho, if you make changes like adding new migrations or changing the queries over [src/database/models](src/database/models) you will need to run:

```bash
cargo sqlx prepare
```

To regenerate those offline queries. Always commit them!

## frontend

The [frontend/](frontend/) is a SvelteKit app that builds to a static site that is then served, in my case, separately from the backend. Its almost full implementation of the backend features into a visually appealing and beautiful neon colored frontend with mah squeaky gelatin buttons.

The Api client is generated by [Orval](https://orval.dev/docs/), which eats the OpenApi docs made by the backend and generates a "type safe" client for the frontend to use. Although, the actual quering is primarely done via [TanStack Query](https://tanstack.com/query/latest/docs/framework/svelte/overview).

the frontend needs an `API_URL` pointing at the backend, and it gets it from either a `frontend/.env` (contents of [frontend/.env.example](frontend/.env.example)) or straight from your shell:

```bash
cp frontend/.env.example frontend/.env
# or
API_URL=http://localhost:8081 yarn dev
```

that env var is the only way it knows where the backend lives.

But a generated Api client means that when the backend changes (adding new handlers, editing what a handler returns...), the frontend will break and you will need to regenerate the Api client. You can do that by running:

```bash
yarn api
```

The other scripts you can also run are:

- `yarn dev` to run the frontend in development mode
- `yarn build` to build the frontend for production
- `yarn preview` to preview the production build
- `yarn fmt` to lint the frontend code
- `yarn check` to check the frontend code for errors
- `yarn update-aaguid` to update the AAGUID list for passkeys. This is used for the icons showed besides the passkey names.

## how can i run this?

If we are talking about running it in development mode:

1. Start the docker compose services

```bash
docker compose up -d # The "-d" flag means "detached", which means it will run in the background. You can omit it if you want to see the logs of the services.
```

2. Apply migrations if not already done

```bash
cargo sqlx migrate run
# or
sqlx migrate run
```

3. Start the backend

```bash
cargo run -- -e development # the "--" argument is to pass the next arguments to the backend executable and not to cargo.
```

4. If you want to start the frontend

```bash
cd frontend
yarn # install packages if not already done
yarn dev
```

5. Poke around

- [http://localhost:8081](http://localhost:8081) - Backend
- [http://localhost:5173](http://localhost:5173) - Frontend
- [http://localhost:8081/scalar](http://localhost:8081/scalar) - Backend Api explorer
- [http://localhost:8081/api-docs/openapi.json](http://localhost:8081/api-docs/openapi.json) - Backend raw OpenApi
- [http://localhost:8025](http://localhost:8025) - Mailpit, where in development you can recieve the emails that the backend sends
- [http://localhost:4983](http://localhost:4983) - Drizzle Studio, a frontend for the database. yes, its literally like Prisma Studio but generic for any database without depending on crap.

## building for deployment

ah, i love deployment. its the worst part of development. but its also the most fun part of it. you get to see your work in action and you get to see how many things break and or suffer for hours because it doesnt work in production. But its fun... half fun.

> [!NOTE]
> In the root of the project, you can see that there is a [flake.nix](flake.nix) file. This is a Nix flake that can be used to run the backend (and only the backend) in, for example, a Nix powered VPS.

### the backend

You can build the backend for production by running:

```bash
# IF you have running a database (for example, the docker compose one)
cargo build --profile dist # "dist" is a custom profile that is defined in the Cargo.toml file. its just aggresive optimizations and stupid things that i threw in there. you can also use the "release" profile if you want to be more standard

# IF you do not have a running database, you can build the backend without connecting to the database by running:
SQLX_OFFLINE=true cargo build --profile dist
```

`SQLX_OFFLINE=true` builds against the query cache in [.sqlx](.sqlx), so it only works while that cache matches the queries. if you touched the queries, run `cargo sqlx prepare` against a live database first (and commit it), or just build without `SQLX_OFFLINE` while the database is reachable.

### the frontend

You can build the frontend for production by running:

```bash
cd frontend
yarn build
```

This will output a static site in the `frontend/build` folder that you can serve with any static file server. You can also use the `yarn preview` command to preview the production build.

This means that `API_URL` must point to the backend url at build time, either in `frontend/.env` or inline like `API_URL=https://auth.example.com yarn build`, otherwise the frontend builds with the wrong url and it wont work at all!!

> [!TIP]
> You can deploy the frontend over, for example, [Cloudflare Pages](https://www.cloudflare.com/products/pages/). This would allow you to let the provider build the frontend for you WHICH means that you can set independently the backend url for the frontend to use in production without worring about the `.env` file.
