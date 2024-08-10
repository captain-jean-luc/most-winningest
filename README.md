# Most-Winningest

This is a little program to scrape the [[Game] Last one to post wins!](https://community.tulpa.info/topic/7356-game-last-one-to-post-wins/) thread and create a leaderboard of "winningness", a score I made up that is simply the sum total of time each person's post was the most recent post (ie for each user, the time between each of their posts and the next).

### Develop

- Install and run a `postgresql` server for development.

- Create a new file called `.env` with the contents:

  ```
  DATABASE_URL="postgresql:///most-winningest-dev"
  ```

  You may need to change the url depending on your setup, see [here](https://www.postgresql.org/docs/15/libpq-connect.html#LIBPQ-CONNSTRING) for more details.

- **If you have the nix build system installed:**
  - Run `nix develop` in the repo dir to enter a dev shell

- **If you do not have the nix build system:**
  - Install [`rustup`](https://rustup.rs).
  - Run `cargo check` to ensure all the dependencies build correctly.
    You will need a c compiler and postgres dev libraries (on debian-ish: `apt install build-essential libpq-dev`)
    The most painful is the postgres library, see [here](https://github.com/sgrif/pq-sys) if you have issues.
  - Run `cargo install diesel_cli`

- Run `diesel setup` to create the DB (if necessary) and run migrations.

- You're all set up! You should be able to simply `cargo build` to build and `cargo run` to build+run

