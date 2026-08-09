use std::{env, fs, io, path::{Path, PathBuf}, process::{Command, Stdio}};

const WRANGLER_VERSION: &str = "4.114.0";
const WORKER_BUILD_VERSION: &str = "0.8.5";
const HTMX_VERSION: &str = "2.0.10";
const RESPONSE_TARGETS_VERSION: &str = "2.0.4";
const PICO_VERSION: &str = "2.1.1";

fn main() {
    if let Err(e) = run() { eprintln!("error: {e}"); std::process::exit(1); }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref().unwrap_or("help") {
        "help" | "--help" | "-h" => { help(); Ok(()) }
        "versions" => { versions(); Ok(()) }
        "bootstrap" => bootstrap(),
        "vendor" => vendor_assets(),
        "dev" => dev(),
        "test" => test(),
        "e2e" => e2e(),
        "verify" => verify(),
        "check" => template_check(),
        "smoke" => template_smoke(),
        "new" => new_app(args.collect()),
        "configure" => configure(args.collect()),
        "auth" => auth(),
        "whoami" => wrangler(&["whoami"]),
        "provision" => provision(),
        "migrate" => migrate(false),
        "migrate-local" => migrate(true),
        "deploy" => deploy(),
        other => Err(format!("unknown command '{other}'. Run: cargo xtask help")),
    }
}

fn help() {
    println!(r#"Rust + HTMX Cloudflare Starter

Canonical commands:
  cargo xtask bootstrap             Install/check prerequisites + vendor pinned browser assets
  cargo xtask vendor                Refresh pinned HTMX/Pico/extension assets
  cargo xtask dev                   Run both Workers locally with persistent D1/R2/Queues
  cargo xtask test                  Run Rust tests + static browser/template checks
  cargo xtask e2e                   Boot local Workers and smoke D1 + Queue + public routes
  cargo xtask verify                fmt + clippy + tests + wasm checks + Worker builds
  cargo xtask new NAME [options]    Create a sibling app from this starter
  cargo xtask configure [options]   Rename/configure this clone in place
  cargo xtask auth                  Show Cloudflare token instructions
  cargo xtask whoami                Verify Cloudflare credentials
  cargo xtask provision             Create/reuse D1, R2, Queue, DLQ and write D1 IDs
  cargo xtask migrate               Apply remote D1 migrations
  cargo xtask deploy                Verify, migrate, deploy jobs then app Worker
  cargo xtask versions              Print pinned major component versions

new/configure options:
  --title "My Product"              Human title (defaults from name)
  --dir ../my-product               Destination for `new`
  --github owner/repo               Rewrite CI badge URLs

Cloudflare deployment environment:
  CLOUDFLARE_API_TOKEN
  CLOUDFLARE_ACCOUNT_ID

Wrangler is Cloudflare's required driver. xtask invokes a pinned Wrangler via npx
when a local `wrangler` binary is not available; Node is tooling only, never an
application runtime or source-language dependency.
"#);
}
fn versions(){println!("workers-rs {WORKER_BUILD_VERSION}\nHTMX {HTMX_VERSION}\nresponse-targets {RESPONSE_TARGETS_VERSION}\nPico CSS {PICO_VERSION}\nMaud 0.27.0\nWrangler {WRANGLER_VERSION}\nRust >=1.97.1");}

fn command_version(name: &str) -> Option<String> {
    let output = Command::new(name).arg("--version").output().ok()?;
    if !output.status.success() { return None; }
    Some(format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)).trim().to_string())
}
fn command_exists(name:&str)->bool { command_version(name).is_some() }
fn run_cmd(mut c:Command)->Result<(),String>{ let display=format!("{c:?}"); let s=c.status().map_err(|e|format!("failed to start {display}: {e}"))?; if s.success(){Ok(())}else{Err(format!("command failed ({s}): {display}"))} }
fn output_cmd(mut c:Command)->Result<String,String>{ let display=format!("{c:?}"); let o=c.output().map_err(|e|format!("failed to start {display}: {e}"))?; let all=format!("{}{}",String::from_utf8_lossy(&o.stdout),String::from_utf8_lossy(&o.stderr)); if o.status.success(){Ok(all)}else{Err(format!("{all}\ncommand failed: {display}"))} }
fn wrangler_command()->Command {
    if command_version("wrangler").is_some_and(|v| v.contains(WRANGLER_VERSION)) {
        return Command::new("wrangler");
    }
    let mut c=Command::new("npx");
    c.args(["--yes", &format!("wrangler@{WRANGLER_VERSION}")]);
    c
}
fn wrangler(args:&[&str])->Result<(),String>{ let mut c=wrangler_command(); c.args(args); run_cmd(c) }
fn wrangler_output(args:&[&str])->Result<String,String>{ let mut c=wrangler_command(); c.args(args); output_cmd(c) }

fn vendor_assets()->Result<(),String>{
    let assets=[
        (format!("https://cdn.jsdelivr.net/npm/htmx.org@{HTMX_VERSION}/dist/htmx.min.js"),"public/vendor/htmx.min.js",format!("version:\"{HTMX_VERSION}\"")),
        (format!("https://cdn.jsdelivr.net/npm/htmx-ext-response-targets@{RESPONSE_TARGETS_VERSION}"),"public/vendor/response-targets.js","response-targets".to_string()),
        (format!("https://cdn.jsdelivr.net/npm/@picocss/pico@{PICO_VERSION}/css/pico.min.css"),"public/vendor/pico.min.css","Pico CSS".to_string()),
    ];
    fs::create_dir_all("public/vendor").map_err(|e|e.to_string())?;
    let client=reqwest::blocking::Client::builder().user_agent("rust-htmx-starter-xtask").build().map_err(|e|e.to_string())?;
    for (url,path,marker) in assets {
        let current=fs::read_to_string(path).unwrap_or_default();
        if current.contains(&marker) && !(path.ends_with("pico.min.css") && current.contains("compatible semantic baseline")) { continue; }
        println!("vendoring {url}");
        let res=client.get(&url).send().map_err(|e|format!("download {url}: {e}"))?;
        if !res.status().is_success(){return Err(format!("download {url}: HTTP {}",res.status()));}
        let body=res.text().map_err(|e|e.to_string())?;
        if !body.contains(&marker){return Err(format!("downloaded asset did not contain expected marker {marker}: {url}"));}
        fs::write(path,body).map_err(|e|e.to_string())?;
    }
    Ok(())
}

fn bootstrap()->Result<(),String>{
    if !command_exists("rustup") || !command_exists("cargo") { return Err("Rust/rustup are required. Install from https://rustup.rs, then rerun cargo xtask bootstrap".into()); }
    let mut target=Command::new("rustup"); target.args(["target","add","wasm32-unknown-unknown"]); run_cmd(target)?;
    if !command_version("worker-build").is_some_and(|v| v.contains(WORKER_BUILD_VERSION)) {
        let mut c=Command::new("cargo");
        c.args(["install","worker-build","--locked","--version",WORKER_BUILD_VERSION,"--force"]);
        run_cmd(c)?;
    }
    if !command_version("wrangler").is_some_and(|v| v.contains(WRANGLER_VERSION)) && !command_exists("npx") {
        return Err(format!("Wrangler {WRANGLER_VERSION} is required. Install Node.js 22+ (tooling only) so xtask can invoke the pinned Wrangler version, or install that exact Wrangler release directly."));
    }
    vendor_assets()?;
    println!("bootstrap ready. Run: cargo xtask dev"); Ok(())
}

fn dev()->Result<(),String>{ bootstrap()?; migrate(true)?; let mut c=wrangler_command(); c.args(["dev","-c","workers/app/wrangler.jsonc","-c","workers/jobs/wrangler.jsonc","--persist-to",".wrangler/state","--port","8787"]); run_cmd(c) }
fn test() -> Result<(), String> {
    let mut tests = Command::new("cargo");
    tests.args([
        "test",
        "-p", "starter-contracts",
        "-p", "starter-domain",
        "-p", "starter-templates",
        "-p", "starter-shared",
        "-p", "xtask",
    ]);
    run_cmd(tests)?;
    template_check()
}
fn verify()->Result<(),String>{
    vendor_assets()?;
    template_check()?;
    let mut fmt=Command::new("cargo"); fmt.args(["fmt","--all","--","--check"]); run_cmd(fmt)?;
    let mut clippy_native = Command::new("cargo");
    clippy_native.args(["clippy", "-p", "xtask", "--all-targets", "--", "-D", "warnings"]);
    run_cmd(clippy_native)?;
    let mut clippy_wasm = Command::new("cargo");
    clippy_wasm.args(["clippy", "--workspace", "--exclude", "xtask", "--target", "wasm32-unknown-unknown", "--", "-D", "warnings"]);
    run_cmd(clippy_wasm)?;
    test()?;
    let mut check=Command::new("cargo");
    check.args(["check","--workspace","--exclude","xtask","--target","wasm32-unknown-unknown"]);
    run_cmd(check)?;
    for dir in ["workers/app", "workers/jobs"] {
        let mut build = Command::new("worker-build");
        build.arg("--release").current_dir(dir);
        run_cmd(build)?;
    }
    if command_exists("node") { for js in ["public/app.js","public/sw.js","public/vendor/response-targets.js"] { let mut n=Command::new("node"); n.args(["--check",js]); run_cmd(n)?; } }
    template_smoke()?;
    e2e()?;
    println!("verification complete");
    Ok(())
}

fn e2e() -> Result<(), String> {
    use std::{thread, time::Duration};

    if !command_exists("worker-build") {
        return Err("worker-build is required for e2e; run cargo xtask bootstrap".into());
    }
    if !command_exists("wrangler") && !command_exists("npx") {
        return Err("Wrangler is required for e2e; run cargo xtask bootstrap".into());
    }

    let state = env::temp_dir().join(format!("rust-htmx-e2e-{}", std::process::id()));
    if state.exists() { fs::remove_dir_all(&state).map_err(|e| e.to_string())?; }
    fs::create_dir_all(&state).map_err(|e| e.to_string())?;

    let (db, _, _) = resource_names()?;
    let mut migration = wrangler_command();
    migration.args(["d1", "migrations", "apply", &db, "-c", "workers/app/wrangler.jsonc", "--local", "--persist-to"])
        .arg(&state);
    run_cmd(migration)?;

    let port = "8799";
    let mut dev = wrangler_command();
    dev.args(["dev", "-c", "workers/app/wrangler.jsonc", "-c", "workers/jobs/wrangler.jsonc", "--persist-to"])
        .arg(&state)
        .args(["--port", port])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let mut child = dev.spawn().map_err(|e| format!("failed to start local Workers: {e}"))?;

    let result = (|| -> Result<(), String> {
        // The app is owner-scoped: private routes require a user context. The
        // e2e uses the dev `x-user-id` placeholder (see docs/AUTH.md).
        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert(
            "x-user-id",
            reqwest::header::HeaderValue::from_static("11111111-1111-4111-8111-111111111111"),
        );
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .default_headers(default_headers)
            .build()
            .map_err(|e| e.to_string())?;
        let base = format!("http://127.0.0.1:{port}");
        let mut ready = false;
        for _ in 0..80 {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                return Err(format!("local Workers exited before becoming ready: {status}"));
            }
            if client.get(format!("{base}/healthz")).send().map(|r| r.status().is_success()).unwrap_or(false) {
                ready = true;
                break;
            }
            thread::sleep(Duration::from_millis(250));
        }
        if !ready { return Err("local Workers did not become ready within 20 seconds".into()); }

        let home = client.get(format!("{base}/")).send().map_err(|e| e.to_string())?.text().map_err(|e| e.to_string())?;
        if !home.contains("Evidence Notes") { return Err("home page smoke failed".into()); }

        let create = client.post(format!("{base}/notes"))
            .form(&[("title", "E2E note"), ("body", "Queue and D1 smoke test")])
            .send().map_err(|e| e.to_string())?;
        if create.status().as_u16() != 303 { return Err(format!("create note expected 303, got {}", create.status())); }

        let home = client.get(format!("{base}/")).send().map_err(|e| e.to_string())?.text().map_err(|e| e.to_string())?;
        let marker = "id=\"note-";
        let start = home.find(marker).ok_or("created note not found in home HTML")? + marker.len();
        let rest = &home[start..];
        let end = rest.find('"').ok_or("created note id was malformed")?;
        let note_id = &rest[..end];

        let summarize = client.post(format!("{base}/notes/{note_id}/summarize")).send().map_err(|e| e.to_string())?;
        if summarize.status().as_u16() != 303 { return Err(format!("summarize expected 303, got {}", summarize.status())); }
        let mut summarized = false;
        for _ in 0..40 {
            let body = client.get(format!("{base}/")).send().map_err(|e| e.to_string())?.text().map_err(|e| e.to_string())?;
            if body.contains("Summary:") { summarized = true; break; }
            thread::sleep(Duration::from_millis(250));
        }
        if !summarized { return Err("Queue consumer did not persist summary within 10 seconds".into()); }

        let publish = client.post(format!("{base}/notes/{note_id}/publish")).send().map_err(|e| e.to_string())?;
        if publish.status().as_u16() != 303 { return Err(format!("publish expected 303, got {}", publish.status())); }
        let public = client.get(format!("{base}/notes/{note_id}.json")).send().map_err(|e| e.to_string())?;
        if !public.status().is_success() { return Err(format!("published JSON route failed: {}", public.status())); }
        let body = public.text().map_err(|e| e.to_string())?;
        if !body.contains("\"status\":\"published\"") { return Err("public JSON did not expose published status".into()); }

        println!("local end-to-end smoke passed");
        Ok(())
    })();

    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(&state);
    result
}

fn template_check()->Result<(),String>{
    let required=["README.md","AGENTS.md","CLAUDE.md","CODEX.md","OMNIAGENT.md","Cargo.toml",".cargo/config.toml","crates/xtask/src/main.rs","public/app.js","public/app.css","public/sw.js","public/manifest.webmanifest","public/vendor/htmx.min.js","public/vendor/response-targets.js","public/vendor/pico.min.css","workers/app/wrangler.jsonc","workers/jobs/wrangler.jsonc","docs/AUTH.md","docs/SECRETS.md",".github/workflows/ci.yml",".github/workflows/security.yml"];
    for f in required { if !Path::new(f).exists(){return Err(format!("required starter file missing: {f}"));} }
    let app=fs::read_to_string("public/app.js").map_err(|e|e.to_string())?;
    if app.contains("DOMContentLoaded") { return Err("app.js must not rely on DOMContentLoaded component listeners; use event delegation/HTMX lifecycle hooks".into()); }
    if !app.contains("htmx.onLoad") || !app.contains("document.addEventListener") { return Err("app.js must demonstrate both HTMX lifecycle-safe initialization and document-level event delegation".into()); }
    let tpl=fs::read_to_string("crates/templates/src/lib.rs").map_err(|e|e.to_string())?;
    for needle in ["hx-ext=\"response-targets\"","details","summary","hx-target-422","method=\"post\"","action=\"/notes\""] { if !tpl.contains(needle){return Err(format!("template best-practice marker missing: {needle}"));} }
    let sw=fs::read_to_string("public/sw.js").map_err(|e|e.to_string())?;
    if sw.contains("'/notes'") || sw.contains("/notes/") { return Err("service worker must not precache workspace note routes".into()); }
    if !sw.contains("SKIP_WAITING") || !app.contains("data-update-pwa") { return Err("PWA must include an explicit update lifecycle".into()); }
    let manifest:serde_json::Value=serde_json::from_str(&fs::read_to_string("public/manifest.webmanifest").map_err(|e|e.to_string())?).map_err(|e|format!("invalid PWA manifest: {e}"))?;
    if manifest["display"]!="standalone" || !manifest["icons"].is_array() || manifest["icons"].as_array().map(|a|a.len()).unwrap_or(0)<2 { return Err("PWA manifest must be standalone with at least 192px and 512px icons".into()); }
    for banned in ["package.json","pnpm-lock.yaml","yarn.lock"] { if Path::new(banned).exists(){return Err(format!("polyglot tooling file must not exist: {banned}"));} }
    fn has_py(path: &Path) -> bool {
        let Ok(entries) = fs::read_dir(path) else { return false; };
        for entry in entries.flatten() {
            let q = entry.path();
            if matches!(q.file_name().and_then(|n| n.to_str()), Some("target" | "build" | ".git" | ".wrangler")) {
                continue;
            }
            if q.is_dir() && has_py(&q) { return true; }
            if q.is_file() && q.extension().and_then(|x| x.to_str()) == Some("py") { return true; }
        }
        false
    }
    if has_py(Path::new(".")){return Err("Python source is intentionally excluded from this starter".into());}
    let readme=fs::read_to_string("README.md").map_err(|e|e.to_string())?;
    for needle in ["cargo xtask dev","cargo xtask deploy","CLOUDFLARE_API_TOKEN","Workers Scripts","D1","R2","Queues","AGENTS.md"] { if !readme.contains(needle){return Err(format!("README missing required guidance: {needle}"));} }
    println!("template checks passed"); Ok(())
}

fn parse_opts(args:Vec<String>, require_name:bool)->Result<(String,String,Option<PathBuf>,Option<String>),String>{
    let mut name=None; let mut title=None; let mut dir=None; let mut github=None; let mut i=0;
    while i<args.len(){ match args[i].as_str(){
        "--title"=>{i+=1; title=args.get(i).cloned();}, "--dir"|"--directory"=>{i+=1; dir=args.get(i).map(PathBuf::from);}, "--github"|"--github-repo"=>{i+=1; github=args.get(i).cloned();}, s if !s.starts_with('-') && name.is_none()=>name=Some(s.to_string()), s=>return Err(format!("unknown option: {s}")), } i+=1; }
    if require_name && name.is_none(){return Err("app name required, e.g. cargo xtask new my-product".into());}
    let n=name.unwrap_or_else(||"rust-htmx-starter".into());
    validate_app_name(&n)?;
    let t=title.unwrap_or_else(||humanize(&n));
    Ok((n,t,dir,github))
}

fn validate_app_name(name: &str) -> Result<(), String> {
    let valid = !name.is_empty()
        && name.len() <= 58
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if valid { Ok(()) } else {
        Err("app name must be 1-58 chars of lowercase letters, digits, and hyphens, without leading/trailing hyphens".into())
    }
}
fn humanize(s:&str)->String{s.split(|c| c == '-' || c == '_').filter(|x|!x.is_empty()).map(|x|{let mut c=x.chars();match c.next(){None=>String::new(),Some(f)=>f.to_uppercase().collect::<String>()+c.as_str()}}).collect::<Vec<_>>().join(" ")}
fn copy_tree(src:&Path,dst:&Path)->io::Result<()> { fs::create_dir_all(dst)?; for ent in fs::read_dir(src)? { let ent=ent?; let p=ent.path(); let name=ent.file_name(); let n=name.to_string_lossy(); if matches!(n.as_ref(), ".git" | "target" | "build" | ".wrangler" | ".cloudflare.env" | ".dev.vars") { continue; } let out=dst.join(name); if p.is_dir(){copy_tree(&p,&out)?}else{fs::copy(&p,&out)?;} } Ok(()) }
fn replace_all(root:&Path,name:&str,title:&str,github:Option<&str>)->Result<(),String>{
    let replacements=[("rust-htmx-starter",name),("Rust + HTMX Starter",title),("cloudflare-rust-htmx-starter",name)];
    fn walk(p:&Path, reps:&[(&str,&str)], github:Option<&str>)->Result<(),String>{ for e in fs::read_dir(p).map_err(|e|e.to_string())?{let e=e.map_err(|e|e.to_string())?;let q=e.path();if q.is_dir(){walk(&q,reps,github)?}else if let Some(ext)=q.extension().and_then(|x|x.to_str()){if ["rs","toml","jsonc","md","json","css","js","yml","yaml","webmanifest","txt","example","sql"].contains(&ext){let Ok(mut s)=fs::read_to_string(&q) else{continue}; for (a,b) in reps{s=s.replace(a,b);} if let Some(g)=github{s=s.replace("OWNER/REPOSITORY",g);} fs::write(&q,s).map_err(|e|e.to_string())?;}}} Ok(())}
    walk(root,&replacements,github)
}
fn reset_cloudflare_ids(root: &Path) -> Result<(), String> {
    for relative in ["workers/app/wrangler.jsonc", "workers/jobs/wrangler.jsonc"] {
        let path = root.join(relative);
        let mut value: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("invalid {}: {e}", path.display()))?;
        value["d1_databases"][0]["database_id"] = serde_json::Value::String("00000000-0000-0000-0000-000000000000".into());
        fs::write(&path, serde_json::to_string_pretty(&value).map_err(|e| e.to_string())? + "\n").map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn new_app(args:Vec<String>)->Result<(),String>{
    let (name,title,dir,github)=parse_opts(args,true)?;
    let dst=dir.unwrap_or_else(||PathBuf::from("..").join(&name));
    if dst.exists(){return Err(format!("destination already exists: {}",dst.display()));}
    copy_tree(Path::new("."),&dst).map_err(|e|e.to_string())?;
    replace_all(&dst,&name,&title,github.as_deref())?;
    reset_cloudflare_ids(&dst)?;
    println!("created {}\nnext:\n  cd {}\n  cargo xtask dev",title,dst.display());
    Ok(())
}
fn configure(args:Vec<String>)->Result<(),String>{ let (name,title,_,github)=parse_opts(args,true)?; replace_all(Path::new("."),&name,&title,github.as_deref())?; println!("configured {title}"); Ok(()) }
fn template_smoke() -> Result<(), String> {
    let dir = env::temp_dir().join(format!("rust-htmx-starter-smoke-{}", std::process::id()));
    if dir.exists() { fs::remove_dir_all(&dir).map_err(|e| e.to_string())?; }
    copy_tree(Path::new("."), &dir).map_err(|e| e.to_string())?;
    replace_all(&dir, "sample-product", "Sample Product", Some("example/sample-product"))?;
    reset_cloudflare_ids(&dir)?;
    let app = fs::read_to_string(dir.join("workers/app/wrangler.jsonc")).map_err(|e| e.to_string())?;
    for expected in ["sample-product-app", "sample-product-db", "sample-product-attachments", "sample-product-jobs"] {
        if !app.contains(expected) { return Err(format!("generator smoke missing {expected}")); }
    }
    if !app.contains("00000000-0000-0000-0000-000000000000") {
        return Err("generator smoke did not reset Cloudflare database id".into());
    }
    if dir.join(".cloudflare.env").exists() || dir.join(".dev.vars").exists() {
        return Err("generator smoke copied credential files".into());
    }
    fs::remove_dir_all(&dir).ok();
    println!("generator smoke passed");
    Ok(())
}

fn auth()->Result<(),String>{ println!(r#"Cloudflare API token

Dashboard → My Profile → API Tokens → Create Token → Create Custom Token

Account permissions (Edit):
  Workers Scripts   deploy app/jobs Workers
  D1                create database + migrations
  Workers R2 Storage create bucket
  Queues            create queue + DLQ

Scope the token to the intended account only. Zone permissions are not needed
unless you later add DNS/routes/custom-domain automation.

Set locally (never put these in .dev.vars or Worker runtime vars):
  export CLOUDFLARE_API_TOKEN='...'
  export CLOUDFLARE_ACCOUNT_ID='...'

Then run:
  cargo xtask whoami

For GitHub deployment workflows, store the same names as repository/environment
secrets. CI/test workflows intentionally do not receive deployment secrets.
"#); Ok(()) }
fn ensure_cloudflare_auth()->Result<(),String>{ if env::var("CLOUDFLARE_API_TOKEN").is_err() && env::var("CI").is_ok(){return Err("CLOUDFLARE_API_TOKEN is required in CI".into());} Ok(()) }
fn read_json_config(path:&str)->Result<serde_json::Value,String>{
    let s=fs::read_to_string(path).map_err(|e|e.to_string())?;
    serde_json::from_str(&s).map_err(|e|format!("invalid {path}: {e}"))
}
fn resource_names()->Result<(String,String,String),String>{
    let v=read_json_config("workers/app/wrangler.jsonc")?;
    let db=v["d1_databases"][0]["database_name"].as_str().ok_or("database_name missing")?.to_string();
    let bucket=v["r2_buckets"][0]["bucket_name"].as_str().ok_or("bucket_name missing")?.to_string();
    let queue=v["queues"]["producers"][0]["queue"].as_str().ok_or("queue missing")?.to_string();
    Ok((db,bucket,queue))
}
fn create_or_reuse(label:&str,args:&[&str])->Result<(),String>{match wrangler_output(args){Ok(o)=>{println!("{o}");Ok(())},Err(e) if e.to_ascii_lowercase().contains("already exists")||e.to_ascii_lowercase().contains("already been taken")=>{println!("{label} already exists; reusing it");Ok(())},Err(e)=>Err(e)}}
fn set_database_id(path:&str,id:&str)->Result<(),String>{
    let mut v=read_json_config(path)?;
    v["d1_databases"][0]["database_id"]=serde_json::Value::String(id.to_string());
    fs::write(path,serde_json::to_string_pretty(&v).map_err(|e|e.to_string())?+"\n").map_err(|e|e.to_string())
}
fn provision()->Result<(),String>{
    ensure_cloudflare_auth()?;
    let (db,bucket,queue)=resource_names()?; let dlq=format!("{queue}-dlq");
    create_or_reuse("D1 database",&["d1","create",&db])?;
    let info=wrangler_output(&["d1","info",&db,"--json"])?;
    let start=info.find('{').unwrap_or(0);
    let v:serde_json::Value=serde_json::from_str(&info[start..]).map_err(|e|format!("cannot parse d1 info json: {e}\n{info}"))?;
    let id=v.get("uuid").or_else(||v.get("database_id")).or_else(||v.get("id")).and_then(|x|x.as_str()).ok_or_else(||format!("D1 id missing from: {v}"))?;
    set_database_id("workers/app/wrangler.jsonc",id)?; set_database_id("workers/jobs/wrangler.jsonc",id)?;
    create_or_reuse("R2 bucket",&["r2","bucket","create",&bucket])?;
    create_or_reuse("Queue",&["queues","create",&queue])?; create_or_reuse("DLQ",&["queues","create",&dlq])?;
    println!("provisioned resources; D1 id written to both configs\nnext: cargo xtask migrate"); Ok(())
}
fn migrate(local:bool)->Result<(),String>{
    let (db,_,_)=resource_names()?;
    let mut c=wrangler_command();
    c.args(["d1","migrations","apply",&db,"-c","workers/app/wrangler.jsonc"]);
    if local {
        c.args(["--local", "--persist-to", ".wrangler/state"]);
    } else {
        ensure_cloudflare_auth()?;
        c.arg("--remote");
    }
    run_cmd(c)
}
fn deploy() -> Result<(), String> {
    ensure_cloudflare_auth()?;
    verify()?;
    provision()?;
    migrate(false)?;
    wrangler(&["deploy", "-c", "workers/jobs/wrangler.jsonc"])?;
    wrangler(&["deploy", "-c", "workers/app/wrangler.jsonc"])?;
    println!("deployment complete");
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name_validation_is_strict() {
        assert!(validate_app_name("my-product-2").is_ok());
        assert!(validate_app_name("My Product").is_err());
        assert!(validate_app_name("-bad").is_err());
        assert!(validate_app_name("bad-").is_err());
    }

    #[test]
    fn humanize_turns_slug_into_title() {
        assert_eq!(humanize("my-product"), "My Product");
    }
}
