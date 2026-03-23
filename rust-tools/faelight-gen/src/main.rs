//! faelight-gen — Forest-Native Password & Secret Generator Suite
//! INT-130: 12 generator types, colored output, TUI menu
//! "Security through randomness. Beauty through color."

use clap::{Parser, Subcommand};
use colored::*;
use rand::Rng;

#[derive(Parser)]
#[command(name = "faelight-gen", about = "🔐 Forest-native secret generator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Random character password
    Random {
        #[arg(short, long, default_value = "32")]
        length: usize,
    },
    /// Diceware passphrase
    Passphrase {
        #[arg(short, long, default_value = "4")]
        words: usize,
    },
    /// UUID v4
    Uuid,
    /// Name-based username
    Username,
    /// Numeric PIN
    Pin {
        #[arg(short, long, default_value = "6")]
        digits: usize,
    },
    /// API key
    Apikey {
        #[arg(short, long, default_value = "sk")]
        prefix: String,
    },
    /// Base64 secret
    Base64 {
        #[arg(short, long, default_value = "32")]
        bytes: usize,
    },
    /// Base32 secret
    Base32 {
        #[arg(short, long, default_value = "20")]
        bytes: usize,
    },
    /// Cryptographic key (hex)
    Cryptokey {
        #[arg(short, long, default_value = "256")]
        bits: usize,
    },
    /// 12-word mnemonic seed phrase
    Seed,
    /// Pronounceable password
    Pronounceable {
        #[arg(short, long, default_value = "12")]
        length: usize,
    },
    /// Session token
    Token {
        #[arg(short, long, default_value = "sess")]
        prefix: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => run_tui(),
        Some(cmd) => run_command(cmd),
    }
}

fn run_command(cmd: Commands) {
    let (label, secret, kind) = match cmd {
        Commands::Random { length } => ("Random Character", gen_random(length), "random"),
        Commands::Passphrase { words } => ("Passphrase", gen_passphrase(words), "passphrase"),
        Commands::Uuid => ("UUID v4", gen_uuid(), "uuid"),
        Commands::Username => ("Username", gen_username(), "username"),
        Commands::Pin { digits } => ("PIN", gen_pin(digits), "pin"),
        Commands::Apikey { prefix } => ("API Key", gen_apikey(&prefix), "apikey"),
        Commands::Base64 { bytes } => ("Base64 Secret", gen_base64(bytes), "base64"),
        Commands::Base32 { bytes } => ("Base32 Secret", gen_base32(bytes), "base32"),
        Commands::Cryptokey { bits } => ("Cryptographic Key", gen_cryptokey(bits), "cryptokey"),
        Commands::Seed => ("Seed Phrase", gen_seed(), "seed"),
        Commands::Pronounceable { length } => {
            ("Pronounceable", gen_pronounceable(length), "pronounceable")
        }
        Commands::Token { prefix } => ("Token", gen_token(&prefix), "token"),
    };
    display_result(label, &secret, kind);
}

fn display_result(label: &str, secret: &str, kind: &str) {
    println!();
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan()
    );
    println!("  🔐 {}", label.bright_white().bold());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan()
    );
    println!();

    // Colored output per character type
    match kind {
        "random" | "pronounceable" | "apikey" | "token" => {
            print!("  ");
            for c in secret.chars() {
                if c.is_ascii_digit() {
                    print!("{}", c.to_string().bright_red().bold());
                } else if c.is_ascii_uppercase() {
                    print!("{}", c.to_string().bright_green().bold());
                } else if c.is_ascii_lowercase() {
                    print!("{}", c.to_string().green());
                } else {
                    print!("{}", c.to_string().yellow().bold());
                }
            }
            println!();
            println!();
            println!("  {} letters  {} numbers  {} symbols", "🟢", "🔴", "🟡");
        }
        "pin" => {
            print!("  ");
            for c in secret.chars() {
                print!("{} ", c.to_string().bright_red().bold());
            }
            println!();
        }
        "passphrase" | "seed" => {
            print!("  ");
            let words: Vec<&str> = secret.split('-').collect();
            let colors = [
                |s: &str| s.bright_green().to_string(),
                |s: &str| s.bright_cyan().to_string(),
                |s: &str| s.bright_yellow().to_string(),
                |s: &str| s.bright_magenta().to_string(),
            ];
            for (i, word) in words.iter().enumerate() {
                let colored = colors[i % colors.len()](word);
                print!("{} ", colored);
            }
            println!();
        }
        _ => {
            println!("  {}", secret.bright_green());
        }
    }

    println!();

    // Strength display
    let entropy = calc_entropy(secret, kind);
    let strength = entropy_to_strength(entropy);
    let bar_filled = ((entropy / 128.0) * 20.0).min(20.0) as usize;
    let bar_empty = 20usize.saturating_sub(bar_filled);
    let bar = format!(
        "{}{}",
        "█".repeat(bar_filled).bright_green(),
        "░".repeat(bar_empty).dimmed()
    );

    println!("  Strength:  {} {}", bar, strength.bold());
    println!("  Entropy:   {:.1} bits", entropy);
    println!("  Type:      {}", label.dimmed());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();
}

fn run_tui() {
    println!();
    println!(
        "{}",
        "  ╭─ 🔐 faelight-gen ──────────────────────────────────╮".bright_cyan()
    );
    println!("  │  Choose your generator:                              │");
    println!("  │                                                      │");
    println!(
        "  │   {}  Random Character    {}  Passphrase",
        "1".bright_white().bold(),
        "2".bright_white().bold()
    );
    println!(
        "  │   {}  UUID               {}  Username",
        "3".bright_white().bold(),
        "4".bright_white().bold()
    );
    println!(
        "  │   {}  PIN                {}  API Key",
        "5".bright_white().bold(),
        "6".bright_white().bold()
    );
    println!(
        "  │   {}  Base64             {}  Base32",
        "7".bright_white().bold(),
        "8".bright_white().bold()
    );
    println!(
        "  │   {}  Crypto Key         {}  Seed Phrase",
        "9".bright_white().bold(),
        "10".bright_white().bold()
    );
    println!(
        "  │   {}  Pronounceable      {}  Token",
        "11".bright_white().bold(),
        "12".bright_white().bold()
    );
    println!("  │                                                      │");
    println!(
        "{}",
        "  ╰──────────────────────────────────────────────────────╯".bright_cyan()
    );
    println!();
    print!("  Select [1-12]: ");

    use std::io::{self, BufRead, Write};
    io::stdout().flush().ok();
    let stdin = io::stdin();
    let line = stdin
        .lock()
        .lines()
        .next()
        .and_then(|l| l.ok())
        .unwrap_or_default();

    let cmd = match line.trim() {
        "1" => Commands::Random { length: 32 },
        "2" => Commands::Passphrase { words: 4 },
        "3" => Commands::Uuid,
        "4" => Commands::Username,
        "5" => Commands::Pin { digits: 6 },
        "6" => Commands::Apikey {
            prefix: "sk".to_string(),
        },
        "7" => Commands::Base64 { bytes: 32 },
        "8" => Commands::Base32 { bytes: 20 },
        "9" => Commands::Cryptokey { bits: 256 },
        "10" => Commands::Seed,
        "11" => Commands::Pronounceable { length: 12 },
        "12" => Commands::Token {
            prefix: "sess".to_string(),
        },
        _ => {
            println!("  {} Invalid choice", "✗".bright_red());
            return;
        }
    };
    run_command(cmd);
}

// ── Generators ────────────────────────────────────────────────────────────────

fn gen_random(length: usize) -> String {
    let charset: Vec<char> =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*"
            .chars()
            .collect();
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| charset[rng.gen_range(0..charset.len())])
        .collect()
}

fn gen_passphrase(words: usize) -> String {
    let wordlist = [
        "forest", "river", "stone", "flame", "cloud", "earth", "wind", "dawn", "dusk", "tide",
        "frost", "bloom", "glade", "creek", "ridge", "vale", "moss", "fern", "bark", "leaf",
        "root", "stem", "vine", "seed", "hawk", "wolf", "bear", "deer", "fox", "owl", "crow",
        "swan", "iron", "silver", "gold", "copper", "amber", "jade", "onyx", "ruby", "swift",
        "quiet", "brave", "wise", "bold", "calm", "keen", "true",
    ];
    let mut rng = rand::thread_rng();
    (0..words)
        .map(|_| wordlist[rng.gen_range(0..wordlist.len())])
        .collect::<Vec<_>>()
        .join("-")
}

fn gen_uuid() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],bytes[1],bytes[2],bytes[3],bytes[4],bytes[5],
        bytes[6],bytes[7],bytes[8],bytes[9],bytes[10],bytes[11],
        bytes[12],bytes[13],bytes[14],bytes[15]
    )
}

fn gen_username() -> String {
    let adjectives = [
        "swift", "quiet", "brave", "wise", "bold", "calm", "keen", "dark", "bright", "silver",
    ];
    let nouns = [
        "falcon", "wolf", "cedar", "river", "stone", "frost", "ember", "vale", "hawk", "pine",
    ];
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(10..99);
    format!(
        "{}_{}_{}",
        adjectives[rng.gen_range(0..adjectives.len())],
        nouns[rng.gen_range(0..nouns.len())],
        num
    )
}

fn gen_pin(digits: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..digits)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect()
}

fn gen_apikey(prefix: &str) -> String {
    let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    let mut rng = rand::thread_rng();
    let key: String = (0..32)
        .map(|_| charset[rng.gen_range(0..charset.len())])
        .collect();
    format!("{}_{}", prefix, key)
}

fn gen_base64(bytes: usize) -> String {
    use base64::Engine;
    let mut rng = rand::thread_rng();
    let raw: Vec<u8> = (0..bytes).map(|_| rng.gen::<u8>()).collect();
    base64::engine::general_purpose::STANDARD.encode(&raw)
}

fn gen_base32(bytes: usize) -> String {
    let mut rng = rand::thread_rng();
    let raw: Vec<u8> = (0..bytes).map(|_| rng.gen::<u8>()).collect();
    base32::encode(base32::Alphabet::RFC4648 { padding: false }, &raw)
}

fn gen_cryptokey(bits: usize) -> String {
    let bytes = bits / 8;
    let mut rng = rand::thread_rng();
    let raw: Vec<u8> = (0..bytes).map(|_| rng.gen::<u8>()).collect();
    raw.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

fn gen_seed() -> String {
    let words = [
        "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract", "absurd",
        "abuse", "access", "accident", "account", "accuse", "achieve", "acid", "acoustic",
        "acquire", "across", "act", "action", "actor", "actual", "adapt", "forest", "river",
        "stone", "flame", "cloud", "earth", "wind", "dawn", "wisdom", "courage", "vision",
        "beacon", "anchor", "bridge", "harbor", "shield", "crystal", "ember", "frost", "glade",
        "haven", "journey", "legend", "mystic",
    ];
    let mut rng = rand::thread_rng();
    (0..12)
        .map(|_| words[rng.gen_range(0..words.len())])
        .collect::<Vec<_>>()
        .join("-")
}

fn gen_pronounceable(length: usize) -> String {
    let consonants = "bcdfghjklmnprstvwxyz";
    let vowels = "aeiou";
    let mut rng = rand::thread_rng();
    let mut result = String::with_capacity(length);
    let mut use_vowel = rng.gen_bool(0.5);
    while result.len() < length {
        if use_vowel {
            let v: Vec<char> = vowels.chars().collect();
            result.push(v[rng.gen_range(0..v.len())]);
        } else {
            let c: Vec<char> = consonants.chars().collect();
            result.push(c[rng.gen_range(0..c.len())]);
        }
        use_vowel = !use_vowel;
    }
    result
}

fn gen_token(prefix: &str) -> String {
    let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    let mut rng = rand::thread_rng();
    let token: String = (0..24)
        .map(|_| charset[rng.gen_range(0..charset.len())])
        .collect();
    format!("{}_{}", prefix, token)
}

// ── Entropy ───────────────────────────────────────────────────────────────────

fn calc_entropy(secret: &str, kind: &str) -> f64 {
    match kind {
        "random" => secret.len() as f64 * 6.0,
        "passphrase" => secret.split('-').count() as f64 * 11.0,
        "pin" => secret.len() as f64 * 3.32,
        "uuid" => 122.0,
        "base64" => (secret.len() as f64 * 6.0).min(256.0),
        "base32" => (secret.len() as f64 * 5.0).min(256.0),
        "cryptokey" => (secret.len() as f64 * 4.0).min(256.0),
        "seed" => secret.split('-').count() as f64 * 11.0,
        "pronounceable" => secret.len() as f64 * 4.0,
        _ => secret.len() as f64 * 5.0,
    }
}

fn entropy_to_strength(entropy: f64) -> ColoredString {
    if entropy >= 128.0 {
        "VERY STRONG".bright_green().bold()
    } else if entropy >= 96.0 {
        "STRONG".green().bold()
    } else if entropy >= 64.0 {
        "GOOD".yellow().bold()
    } else if entropy >= 40.0 {
        "FAIR".bright_yellow().bold()
    } else {
        "WEAK".bright_red().bold()
    }
}
