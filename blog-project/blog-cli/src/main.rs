use anyhow::Result;
use blog_client::BlogClient;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "blog-cli")]
#[command(about = "CLI клиент для блога")]
struct Cli {
    #[arg(long, short = 'g', help = "Использовать gRPC вместо HTTP")]
    grpc: bool,

    #[arg(long, short = 's', help = "Адрес сервера")]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Register {
        #[arg(long, short)]
        username: String,
        #[arg(long, short)]
        email: String,
        #[arg(long, short)]
        password: String,
    },
    Login {
        #[arg(long, short)]
        username: String,
        #[arg(long, short)]
        password: String,
    },
    Create {
        #[arg(long, short)]
        title: String,
        #[arg(long, short)]
        content: String,
    },
    Get {
        #[arg(long, short)]
        id: i64,
    },
    Update {
        #[arg(long, short)]
        id: i64,
        #[arg(long, short)]
        title: Option<String>,
        #[arg(long, short)]
        content: Option<String>,
    },
    Delete {
        #[arg(long, short)]
        id: i64,
    },
    List {
        #[arg(long, short, default_value = "10")]
        limit: i64,
        #[arg(long, short, default_value = "0")]
        offset: i64,
    },
}

fn token_file_path() -> PathBuf {
    std::env::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".blog_token")
}

fn load_token() -> Option<String> {
    std::fs::read_to_string(token_file_path()).ok()
}

fn save_token(token: &str) -> Result<()> {
    std::fs::write(token_file_path(), token)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    // Определяем адрес по умолчанию
    let addr = cli.server.unwrap_or_else(|| {
        if cli.grpc {
            "http://127.0.0.1:50051".into()
        } else {
            "http://127.0.0.1:3000".into()
        }
    });

    // Создаём клиент через новые конструкторы
    let mut client = if cli.grpc {
        BlogClient::new_grpc(addr).await?
    } else {
        BlogClient::new_http(addr).await?
    };

    // Загружаем сохранённый токен
    if let Some(token) = load_token() {
        client.set_token(token);
    }

    match cli.command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let resp = client.register(&username, &email, &password).await?;
            save_token(&resp.token)?;
            println!(
                "Registered as {} (ID: {})",
                resp.user.username, resp.user.id
            );
        }
        Commands::Login { username, password } => {
            let resp = client.login(&username, &password).await?;
            save_token(&resp.token)?;
            println!(
                " Logged in as {} (ID: {})",
                resp.user.username, resp.user.id
            );
        }
        Commands::Create { title, content } => {
            let post = client.create_post(&title, &content).await?;
            println!("Post created: #{} - {}", post.id, post.title);
        }
        Commands::Get { id } => {
            let post = client.get_post(id).await?;
            println!("\n📝 Post #{} by author #{}", post.id, post.author_id);
            println!("📌 {}", post.title);
            println!("📄 {}", post.content);
            println!("🕐 Created: {}", post.created_at);
        }
        Commands::Update { id, title, content } => {
            let post = client
                .update_post(id, title.as_deref(), content.as_deref())
                .await?;
            println!("Post #{} updated", post.id);
        }
        Commands::Delete { id } => {
            client.delete_post(id).await?;
            println!("Post #{} deleted", id);
        }
        Commands::List { limit, offset } => {
            let list = client.list_posts(Some(limit), Some(offset)).await?;
            println!(
                "\n📋 Posts (showing {}-{} of {}):",
                offset,
                offset + list.posts.len() as i64,
                list.total
            );
            for post in &list.posts {
                println!("  #{} - {} (by #{})", post.id, post.title, post.author_id);
            }
        }
    }

    Ok(())
}
