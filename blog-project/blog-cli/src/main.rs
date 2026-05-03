// blog-cli/src/main.rs
use clap::{Parser, Subcommand};
use blog_client::{BlogClient, Transport};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "blog-cli")]
#[command(about = "CLI клиент для блога", long_about = None)]
struct Cli {
    /// Использовать gRPC вместо HTTP
    #[arg(long, short = 'g')]
    grpc: bool,
    
    /// Адрес сервера (по умолчанию: http://localhost:8080 или grpc://localhost:50051)
    #[arg(long, short = 's')]
    server: Option<String>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Регистрация нового пользователя
    Register {
        #[arg(long, short)]
        username: String,
        #[arg(long, short)]
        email: String,
        #[arg(long, short)]
        password: String,
    },
    
    /// Вход в систему
    Login {
        #[arg(long, short)]
        username: String,
        #[arg(long, short)]
        password: String,
    },
    
    /// Создание поста (требует авторизации)
    Create {
        #[arg(long, short)]
        title: String,
        #[arg(long, short)]
        content: String,
    },
    
    /// Получение поста по ID
    Get {
        #[arg(long, short)]
        id: i64,
    },
    
    /// Обновление поста (требует авторизации, только автор)
    Update {
        #[arg(long, short)]
        id: i64,
        #[arg(long, short)]
        title: Option<String>,
        #[arg(long, short)]
        content: Option<String>,
    },
    
    /// Удаление поста (требует авторизации, только автор)
    Delete {
        #[arg(long, short)]
        id: i64,
    },
    
    /// Список постов с пагинацией
    List {
        #[arg(long, short, default_value = "10")]
        limit: i64,
        #[arg(long, short, default_value = "0")]
        offset: i64,
    },
}

/// Путь к файлу для сохранения токена
fn token_file_path() -> PathBuf {
    std::env::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".blog_token")
}

/// Загрузка сохранённого токена
fn load_token() -> Option<String> {
    std::fs::read_to_string(token_file_path()).ok()
}

/// Сохранение токена
fn save_token(token: &str) -> Result<()> {
    std::fs::write(token_file_path(), token)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    
    // Определяем адрес сервера
    let addr = cli.server.unwrap_or_else(|| {
        if cli.grpc {
            "http://127.0.0.1:50051".into()
        } else {
            "http://127.0.0.1:3000".into()
        }
    });
    
    // Создаём клиент с выбранным транспортом
    let transport = if cli.grpc {
        Transport::Grpc(addr)
    } else {
        Transport::Http(addr)
    };
    
    let mut client = BlogClient::new(transport).await?;
    
    // Пытаемся загрузить сохранённый токен
    if let Some(token) = load_token() {
        client.set_token(token);
    }
    
    // Выполняем команду
    match cli.command {
        Commands::Register { username, email, password } => {
            let resp = client.register(&username, &email, &password).await?;
            save_token(&resp.token)?;
            println!("✅ Registered as {} (ID: {})", resp.user.username, resp.user.id);
            println!("🔑 Token saved to ~/.blog_token");
        }
        
        Commands::Login { username, password } => {
            let resp = client.login(&username, &password).await?;
            save_token(&resp.token)?;
            println!("✅ Logged in as {} (ID: {})", resp.user.username, resp.user.id);
            println!("🔑 Token saved to ~/.blog_token");
        }
        
        Commands::Create { title, content } => {
            let post = client.create_post(&title, &content).await?;
            println!("✅ Post created: #{} - {}", post.id, post.title);
        }
        
        Commands::Get { id } => {
            let post = client.get_post(id).await?;
            println!("\n📝 Post #{} by author #{}", post.id, post.author_id);
            println!("📌 {}", post.title);
            println!("📄 {}", post.content);
            println!("🕐 Created: {}", post.created_at);
        }
        
        Commands::Update { id, title, content } => {
            let post = client.update_post(id, title.as_deref(), content.as_deref()).await?;
            println!("✅ Post #{} updated", post.id);
        }
        
        Commands::Delete { id } => {
            client.delete_post(id).await?;
            println!("✅ Post #{} deleted", id);
        }
        
        Commands::List { limit, offset } => {
            let list = client.list_posts(Some(limit), Some(offset)).await?;
            println!("\n📋 Posts (showing {}-{} of {}):", offset, offset + list.posts.len() as i64, list.total);
            for post in &list.posts {
                println!("  #{} - {} (by #{})", post.id, post.title, post.author_id);
            }
        }
    }
    
    Ok(())
}