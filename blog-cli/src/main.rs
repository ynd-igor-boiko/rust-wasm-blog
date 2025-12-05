use anyhow::{Context, Result};
use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "blog-cli", about = "Blog CLI client")]
struct Args {
    #[arg(long)]
    grpc: bool,

    #[arg(long, default_value = "http://localhost:3000")]
    server: String,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Register {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        password: String,
    },
    Login {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
    },
    Create {
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        content: String,
    },
    Get { id: i64 },
    Update {
        id: i64,
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        content: String,
    },
    Delete { id: i64 },
    List {
        #[arg(short, long, default_value = "10")]
        limit: i32,
        #[arg(short, long, default_value = "0")]
        offset: i32,
    },
}

fn token_path() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| ".".into()).join(".blog_token")
}

fn save_token(t: &str) -> Result<()> {
    fs::write(token_path(), t).context("failed to save token")
}

fn load_token() -> Option<String> {
    fs::read_to_string(token_path()).ok().filter(|s| !s.is_empty())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let transport = if args.grpc {
        let addr = if args.server.contains(":3000") {
            args.server.replace(":3000", ":50051")
        } else {
            args.server
        };
        Transport::Grpc(addr)
    } else {
        Transport::Http(args.server)
    };

    let mut client = BlogClient::new(transport).await?;
    if let Some(tok) = load_token() {
        client.set_token(tok);
    }

    match args.cmd {
        Cmd::Register { username, email, password } => {
            let resp = client.register(&username, &email, &password).await?;
            save_token(&resp.token)?;
            println!("registered as {} (id={})", resp.user.username, resp.user.id);
        }

        Cmd::Login { username, password } => {
            let resp = client.login(&username, &password).await?;
            save_token(&resp.token)?;
            println!("logged in as {}", resp.user.username);
        }

        Cmd::Create { title, content } => {
            let p = client.create_post(&title, &content).await?;
            println!("created post #{}: {}", p.id, p.title);
        }

        Cmd::Get { id } => {
            let p = client.get_post(id).await?;
            println!("#{} - {}", p.id, p.title);
            println!("by {} at {}", p.author_username, p.created_at);
            println!();
            println!("{}", p.content);
        }

        Cmd::Update { id, title, content } => {
            let p = client.update_post(id, &title, &content).await?;
            println!("updated post #{}", p.id);
        }

        Cmd::Delete { id } => {
            client.delete_post(id).await?;
            println!("deleted post #{}", id);
        }

        Cmd::List { limit, offset } => {
            let resp = client.list_posts(limit, offset).await?;
            if resp.posts.is_empty() {
                println!("no posts found");
                return Ok(());
            }
            println!("{} posts (total: {})\n", resp.posts.len(), resp.total);
            for p in resp.posts {
                println!("#{} {} (by {})", p.id, p.title, p.author_username);
            }
        }
    }

    Ok(())
}
