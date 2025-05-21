Below is an example of how you might break down the rewrite project into small, test-driven steps and produce iterative prompts for a code-generation tool. This strategy emphasizes incremental progress, early testing, and no orphaned code. Feel free to adapt as needed.

---

## 1. High-Level Blueprint

**Goal**: Re-implement the Python-based Transparent Classroom Photos Grabber in Rust.

1. **Initialize Project**

   * Create a new Cargo project.
   * Set up the crate structure (modules for config, client, caching, etc.).

2. **Configuration Management**

   * Use Rust’s environment variable approach (`dotenv` crate).
   * Provide an optional config file approach with Serde (JSON or TOML).
   * Expose a `Config` struct to hold application-wide settings.

3. **Logging & Error Handling**

   * Establish a standard logging pattern (e.g., `log`, `env_logger`, or `tracing`).
   * Add basic error types (`thiserror` crate or a custom error enum).

4. **HTTP Client & Basic Auth**

   * Use `reqwest` for HTTP calls (blocking or async).
   * Implement a minimal “login” functionality to replicate the Python `_login` approach.

5. **Caching**

   * Store JSON responses on disk with a timestamp (via `serde_json`).
   * Provide logic to read from cache if valid, else re-fetch.
   * Include tests for caching edge cases (expired cache, missing files, no disk space, etc.).

6. **HTML Parsing**

   * Use `scraper` or similar to parse the returned HTML, e.g., extracting CSRF tokens.
   * Write a few tests for parsing.

7. **Rate Limiting**

   * Implement a rate-limited approach to requests (if needed).
   * Create basic tests confirming we do not exceed request frequency.

8. **Download & Metadata**

   * Decide on approach: either wrap `exiftool` calls or use a pure-Rust library (e.g., `rexif` for EXIF).
   * Add tests to confirm embedded metadata.
   * Handle image format conversions if needed (like HEIF to JPEG).

9. **Integration & CLI**

   * Build out a main function that pulls the flow together:

     1. Load config.
     2. Login.
     3. Crawl pages.
     4. Download & embed each photo.
   * Provide an end-to-end integration test.

---

## 2. Break into Iterative Chunks

Below is one possible breakdown into **eight** chunks, each chunk containing a small set of tasks you can implement, test, and refine before moving on.

1. **Chunk A**: Project Setup & Basic Tests

   * Create new Cargo project.
   * Set up a basic module structure.
   * Write a trivial test to ensure that the project builds and runs.

2. **Chunk B**: Configuration Loading

   * Add `dotenv` + `serde` + `serde_json` crates.
   * Create a `Config` struct in `config.rs`.
   * Load from environment variables (with fallback to file).
   * Write tests that confirm env variables are read correctly.

3. **Chunk C**: Logging & Error Handling

   * Add a `logger` module or just set up a global logger via `env_logger`.
   * Define a custom `Error` enum or use `thiserror`.
   * Write tests that confirm logs are emitted properly (smoke tests).

4. **Chunk D**: Basic HTTP & Session

   * Add `reqwest` (blocking or async).
   * Initialize a “client” struct to store cookies (session).
   * Write tests for a trivial HTTP GET call with a mock server if needed.

5. **Chunk E**: Login Flow

   * Implement the `_login` logic: fetch CSRF token, post credentials, verify success.
   * Write tests with a local test server or by mocking.

6. **Chunk F**: Caching

   * Implement the caching logic (read JSON from disk, check timestamps).
   * Use `serde_json` to store responses.
   * Write tests for cache misses, hits, expiration.

7. **Chunk G**: Crawling & HTML Parsing

   * Add the code that iterates pages and uses `scraper` or similar to parse the data.
   * Write tests that parse the HTML content for things like author, date, photo URLs.

8. **Chunk H**: Download, Metadata, & Integration

   * Implement downloading photos, embedding metadata (via external tool calls or Rust library).
   * Ensure file renaming, timestamps, etc.
   * Write final integration tests that confirm end-to-end behavior.

---

## 3. Further Subdivision into Smaller Steps

Here’s an example of how you might subdivide one of these chunks into more granular tasks. We’ll pick **Chunk E** (Login Flow) as an example:

1. **E1** - Add an integration test that attempts a “login” function call.
2. **E2** - Create a `login()` stub in the client module that just returns `Ok(())`.
3. **E3** - Fill out the code to do a basic GET request for the login page, returning the raw HTML.
4. **E4** - Parse the HTML, extract the CSRF token (return an error if not found).
5. **E5** - Send the POST request with credentials and token.
6. **E6** - Return an error if login fails or if the server complains.
7. **E7** - Confirm in the test that a success message or a correct cookie indicates we’re logged in.

You would do similar breakdowns for caching, crawling, etc. Each step is small enough to test quickly and safely.

---

## 4. Final Prompts for Code Generation

Below is a fictional example of how you might provide prompts to a code-generation LLM. Each prompt is designed to be self-contained and build on prior steps. They’re labeled with triple-backticks for clarity. Adjust as needed for your actual codebase and preference.

---

### **Prompt A1**: *Initialize Cargo Project and Basic Tests* (COMPLETED)

```
You are an AI coding assistant. We are building a Rust project called "transparent-classroom-grabber".
First, create a new Cargo library project with the following files:

1. Cargo.toml
2. src/lib.rs
3. tests/smoke_test.rs

The library should compile and pass a trivial test in `tests/smoke_test.rs`.

Please provide the complete file contents, with any necessary placeholders (like version, authors, etc.).
Use the 2021 edition of Rust and no default features.
```

---

### **Prompt A2**: *Config Module & Tests*

```
We have a Rust project with a library. Now add a new file `src/config.rs` that defines a `Config` struct.
It should have:
- `email: String`
- `password: String`
- `school_id: u32`
- `child_id: u32`

We want to load these from environment variables `TC_EMAIL`, `TC_PASSWORD`, `SCHOOL`, and `CHILD`.
If they are missing, return an error.
Use `dotenv` and `std::env::var`.
Define a function `Config::from_env() -> Result<Self, ConfigError>` that loads them.

Then, in `tests/config_test.rs`, write unit tests that check:
1) We can load a valid config from environment variables.
2) Missing environment variable triggers an error.

Provide the full code for `src/config.rs`, the `ConfigError`, plus the tests in `tests/config_test.rs`.
```

---

### **Prompt A3**: *Logging & Error Handling* (COMPLETED)

```
We have our project with a `Config` struct. Now we want to add logging and an error type for the whole app.
1) Create `src/error.rs` with a custom enum `AppError` using `thiserror`.
2) Integrate logging via `env_logger` in `lib.rs`’s `init()` function, which we’ll call from tests or main.
   - `init()` should just initialize `env_logger` once.
3) Modify the `Config::from_env()` function to return `AppError` instead of `ConfigError`.

Update any references to `ConfigError` in `tests/config_test.rs`.
Show all changed files.
```

---

### **Prompt A4**: *HTTP Client & Basic Testing* (COMPLETED)

```
We have config and logging in place. Next, create a new file `src/client.rs` with a `Client` struct that stores:
- A `reqwest::blocking::Client`
- A reference to our `Config`

Create a constructor `Client::new(config: Config) -> Self`.
In that constructor, build a `reqwest::blocking::Client` that uses a cookie store.

In `tests/client_test.rs`, write tests to confirm we can instantiate `Client`.
For now, just check that creating the `Client` doesn’t panic.
```

---

### **Prompt A5**: *Login Flow (GET & POST with CSRF)* (COMPLETED)

```
Now we implement the login flow.
In `src/client.rs`, add:
1) A method `fn login(&self) -> Result<(), AppError>`.
2) It should:
   - GET the sign_in page
   - Parse out the CSRF token from a meta tag (assume name="csrf-token")
   - POST credentials + token to the same page
   - Return an error if login appears to fail.

Add a new test in `tests/client_test.rs` that mocks or fakes the sign_in page.
We can do that with the `mockito` crate.
Implement the test so that it expects:
   - GET to `/souls/sign_in`, returns a page with a token
   - POST to the same endpoint, returns success
   - We confirm `login()` returns `Ok(())`

Provide complete updated code for `client.rs` and `client_test.rs`.
```

---

### **Prompt A6**: *Caching with JSON on Disk* (COMPLETED)

```
Next, we add caching.
Create `src/cache.rs` with:
- A function `read_cache(path: &Path) -> Result<Option<CacheData>, AppError>`
- A function `write_cache(path: &Path, data: &CacheData) -> Result<(), AppError>`

`CacheData` is a struct with:
   - `timestamp: SystemTime` (when it was written)
   - `payload: Vec<Post>` (just an example structure for storing posts)

Include logic to skip loading if the cache is expired (older than some threshold from config).
Add unit tests in `tests/cache_test.rs` that confirm:
   - Writing + reading works
   - Expired cache returns `Ok(None)`

Show full code for `cache.rs`, `cache_test.rs`, plus any changes to your error types or Cargo.toml.
```

---

### **Prompt A7**: *Crawling & HTML Parsing*

```
In `src/client.rs`, add:
- `fn get_posts(&self, page: u32) -> Result<Vec<Post>, AppError>`
   - Make an HTTP GET request to the Transparent Classroom endpoint, parse JSON, return a vector of `Post`.
   - If needed, parse HTML if the endpoint returns HTML, or just parse JSON if it returns JSON.
   - For now, just return an empty vector or a mocked example.

Add a new test in `tests/client_test.rs`:
- Mocks the /posts.json endpoint
- Returns sample JSON with a couple of “Post” items
- Confirms `get_posts()` returns them

For HTML parsing, if you need to parse HTML from the response, use the `scraper` crate.
Show all relevant code.
```

---

### **Prompt A8**: *Download Photos, Metadata, Integration*

```
Now we tie it all together:
1) Implement `fn download_photo(&self, post: &Post) -> Result<(), AppError>` that:
   - Builds a download URL from the post info
   - GET the photo bytes
   - Writes them to disk (maybe a `photos/` folder)
   - Optionally calls an external command (exiftool) or a Rust library to embed metadata

2) In `tests/client_test.rs`, write an integration test that:
   - Mocks a photo download
   - Confirms the photo file is written
   - (If possible) checks that metadata is embedded

3) Add a `bin/main.rs` that:
   - Loads config
   - Inits logging
   - Creates client
   - Logs in
   - Fetches some posts
   - Downloads photos

Show all changed or new files, ensuring the entire application can now be run end-to-end.
```

---

## 5. Final Note

With this structure, each **Prompt** represents a small deliverable for the LLM to generate code. You test it, refine if necessary, then move on to the next prompt. This ensures no orphaned code and keeps the project in a consistently buildable and testable state.
