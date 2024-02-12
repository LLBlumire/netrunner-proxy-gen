use clap::Parser;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};
use tokio::fs;

#[derive(Parser)]
struct Opt {
    #[arg()]
    card_dir: PathBuf,

    #[arg(
        long,
        default_value = "https://access.nullsignal.games/Gateway/English/English/SystemGatewayEnglish-A4%20Printable%20Sheets%201x.pdf"
    )]
    sg: String,

    #[arg(
        long,
        default_value = "https://access.nullsignal.games/Update/english/English/SystemUpdate2021English-A4%20Printable%20Sheets%201x.pdf"
    )]
    su: String,

    #[arg(
        long,
        default_value = "https://access.nullsignal.games/TAI/EnglishPNP/TheAutomataInitiativeEnglish-A4%20Printable%20Sheets%201x.pdf"
    )]
    tai: String,

    #[arg(
        long,
        default_value = "https://nullsignal.games/wp-content/uploads/2022/12/ParhelionEnglish-A4-Printable-Sheets-1x-1.pdf"
    )]
    ph: String,

    #[arg(
        long,
        default_value = "https://nullsignal.games/wp-content/uploads/2022/07/Midnight-Sun-Final-PNP-A4-English-1x.pdf"
    )]
    ms: String,

    #[arg(short)]
    deck: Vec<String>,

    #[arg(long)]
    include_basic_actions: bool,

    #[arg(long)]
    include_marks: bool,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    fs::create_dir_all(&opt.card_dir).await.unwrap();

    acquire_system_gateway_set(&opt.card_dir.join("sg"), &opt.sg).await;
    acquire_system_update_set(&opt.card_dir.join("su21"), &opt.su).await;
    acquire_the_automata_initiative_set(&opt.card_dir.join("tai"), &opt.tai).await;
    acquire_midnight_sun_set(&opt.card_dir.join("ms"), &opt.ms).await;
    acquire_parhelion_set(&opt.card_dir.join("ph"), &opt.ph).await;

    build_documents(
        &opt.card_dir,
        opt.deck,
        opt.include_basic_actions,
        opt.include_marks,
    )
    .await;
}

async fn build_documents(
    path: &Path,
    decks: Vec<String>,
    include_basic_actions: bool,
    include_marks: bool,
) {
    let mut document = String::new();
    document.push_str("<!DOCTYPE html>\n");
    document.push_str("<html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>PDF</title><style>*,::after,::before{margin:0;padding:0;min-width:0}.page{width:210mm;height:297mm;display:grid;place-items:center}.imgs{display:grid;grid-template-columns:2.5in 2.5in 2.5in;grid-template-rows:3.5in 3.5in 3.5in;gap:0.5pt}img{width:100%;height:100%}</style></head><body><div class=\"page\"><div class=\"imgs\">");
    let mut index = 0;
    for deck in decks.iter() {
        let deck_data = get_json_cached(
            &path.join("cache"),
            &format!("https://netrunnerdb.com/api/2.0/public/deck/{}", deck),
        )
        .await;
        let cards = &deck_data["data"][0]["cards"].as_object().unwrap();
        for (card, count) in cards.iter() {
            let count = count.as_number().unwrap().as_i64().unwrap();
            let card_data = get_json_cached(
                &path.join("cache"),
                &format!("https://netrunnerdb.com/api/2.0/public/card/{}", card),
            )
            .await;
            let card_pack = dbg!(dbg!(&card_data["data"])[0]["pack_code"].as_str().unwrap());
            let card_position = card_data["data"][0]["position"]
                .as_number()
                .unwrap()
                .as_i64()
                .unwrap();
            for _ in 0..count {
                document.push_str(&format!(
                    "<img src=\"{}/cut/c-{:>03}.png\" />",
                    card_pack, card_position
                ));
                if index % 9 == 8 {
                    document.push_str("</div></div><div class=\"page\"><div class=\"imgs\">");
                }
                index += 1;
            }
        }
    }
    if include_basic_actions {
        for card_position in 78..=79 {
            document.push_str(&format!(
                "<img src=\"sg/cut/c-{:>03}.png\" />",
                card_position
            ));
            if index % 9 == 8 {
                document.push_str("</div></div><div class=\"page\"><div class=\"imgs\">");
            }
            index += 1;
        }
    }
    if include_marks {
        for card_position in 66..=68 {
            document.push_str(&format!(
                "<img src=\"ms/cut/c-{:>03}.png\" />",
                card_position
            ));
            if index % 9 == 8 {
                document.push_str("</div></div><div class=\"page\"><div class=\"imgs\">");
            }
            index += 1;
        }
    }
    document.push_str("</body></html>");

    let name = decks.join("_");
    fs::write(path.join(format!("{}.html", name)), document)
        .await
        .unwrap();
}

async fn get_json_cached(path: &Path, url: &str) -> serde_json::Value {
    let hash = md5::compute(url);
    let store = format!("{:?}.json", hash);
    let store = path.join(store);
    create_dir_all(path).unwrap();
    if let Ok(read) = fs::read(&store).await {
        return serde_json::from_slice(&read).unwrap();
    }
    let json: serde_json::Value = reqwest::get(url).await.unwrap().json().await.unwrap();
    fs::write(&store, json.to_string()).await.unwrap();
    json
}

async fn acquire_system_gateway_set(path: &Path, sg: &str) {
    let download_path = download_set_pdf(sg, &path.join("download")).await;
    let mut extracted = extract_images(&download_path, &path.join("extract")).await;
    let mut good_images = Vec::new();
    'entries: while let Ok(Some(entry)) = extracted.next_entry().await {
        let file_name = entry.file_name().into_string().unwrap();
        if file_name.starts_with("x-000") {
            continue;
        }
        if file_name.starts_with("x-020") {
            continue;
        }
        if file_name.starts_with("x-022") {
            continue;
        }
        for n in 1..=23 {
            if n % 2 != 0 && file_name.starts_with(&format!("x-{:>03}", n)) {
                continue 'entries;
            }
        }
        good_images.push(entry.path());
    }
    let good_images = good_images.iter().map(|n| n.as_path());
    let mut cropped = crop_images(good_images, &path.join("crop"), 2233, 3093, 76, 76).await;
    let mut cropped_images = Vec::new();
    while let Ok(Some(entry)) = cropped.next_entry().await {
        cropped_images.push(entry.path())
    }
    let cropped_images = cropped_images.iter().map(|n| n.as_path());
    cutout_images(
        cropped_images,
        &path.join("cut"),
        &[
            [744, 1031, 0, 0],
            [744, 1031, 745, 0],
            [744, 1031, 1489, 0],
            [744, 1031, 0, 1031],
            [744, 1031, 745, 1031],
            [744, 1031, 1489, 1031],
            [744, 1031, 0, 2062],
            [744, 1031, 745, 2062],
            [744, 1031, 1489, 2062],
        ],
    )
    .await;
}

async fn acquire_system_update_set(path: &Path, su: &str) {
    let download_path = download_set_pdf(su, &path.join("download")).await;
    let mut extracted = extract_images(&download_path, &path.join("extract")).await;
    let mut good_images = Vec::new();
    'entries: while let Ok(Some(entry)) = extracted.next_entry().await {
        let file_name = entry.file_name().into_string().unwrap();
        if file_name.starts_with("x-000") {
            continue;
        }
        for n in 1..=21 {
            if n % 2 != 0 && file_name.starts_with(&format!("x-{:>03}", n)) {
                continue 'entries;
            }
        }
        good_images.push(entry.path());
    }
    let good_images = good_images.iter().map(|n| n.as_path());
    let mut cropped = crop_images(good_images, &path.join("crop"), 2233, 3093, 76, 76).await;
    let mut cropped_images = Vec::new();
    while let Ok(Some(entry)) = cropped.next_entry().await {
        cropped_images.push(entry.path())
    }
    let cropped_images = cropped_images.iter().map(|n| n.as_path());
    cutout_images(
        cropped_images,
        &path.join("cut"),
        &[
            [744, 1031, 0, 0],
            [744, 1031, 745, 0],
            [744, 1031, 1489, 0],
            [744, 1031, 0, 1031],
            [744, 1031, 745, 1031],
            [744, 1031, 1489, 1031],
            [744, 1031, 0, 2062],
            [744, 1031, 745, 2062],
            [744, 1031, 1489, 2062],
        ],
    )
    .await;
}

async fn acquire_the_automata_initiative_set(path: &Path, tai: &str) {
    let download_path = download_set_pdf(tai, &path.join("download")).await;
    let mut extracted = extract_images(&download_path, &path.join("extract")).await;
    let mut good_images = Vec::new();
    'entries: while let Ok(Some(entry)) = extracted.next_entry().await {
        let file_name = entry.file_name().into_string().unwrap();
        if file_name.starts_with("x-000") {
            continue;
        }
        if file_name.starts_with("x-018") {
            continue;
        }
        if file_name.starts_with("x-020") {
            continue;
        }
        for n in 1..=21 {
            if n % 2 != 0 && file_name.starts_with(&format!("x-{:>03}", n)) {
                continue 'entries;
            }
        }
        good_images.push(entry.path());
    }
    let good_images = good_images.iter().map(|n| n.as_path());
    let mut cropped = crop_images(good_images, &path.join("crop"), 2233, 3093, 76, 76).await;
    let mut cropped_images = Vec::new();
    while let Ok(Some(entry)) = cropped.next_entry().await {
        cropped_images.push(entry.path())
    }
    let cropped_images = cropped_images.iter().map(|n| n.as_path());
    cutout_images(
        cropped_images,
        &path.join("cut"),
        &[
            [744, 1031, 0, 0],
            [744, 1031, 745, 0],
            [744, 1031, 1489, 0],
            [744, 1031, 0, 1031],
            [744, 1031, 745, 1031],
            [744, 1031, 1489, 1031],
            [744, 1031, 0, 2062],
            [744, 1031, 745, 2062],
            [744, 1031, 1489, 2062],
        ],
    )
    .await;
}

async fn acquire_midnight_sun_set(path: &Path, ms: &str) {
    let download_path = download_set_pdf(ms, &path.join("download")).await;
    let mut extracted = extract_images(&download_path, &path.join("extract")).await;
    let mut good_images = Vec::new();
    'entries: while let Ok(Some(entry)) = extracted.next_entry().await {
        let file_name = entry.file_name().into_string().unwrap();
        for i in 0..=9 {
            if file_name.starts_with(&format!("x-{:>03}", i)) {
                continue 'entries;
            }
        }
        if file_name.starts_with("x-078") {
            continue;
        }
        if file_name.starts_with("x-079") {
            continue;
        }
        good_images.push(entry.path());
    }
    let good_images = good_images.iter().map(|n| n.as_path());
    shift_offset_cards(good_images, &path.join("cut"), 3, 1, 68).await;
}

async fn acquire_parhelion_set(path: &Path, ph: &str) {
    let download_path = download_set_pdf(ph, &path.join("download")).await;
    let mut extracted = extract_images(&download_path, &path.join("extract")).await;
    let mut good_images = Vec::new();
    'entries: while let Ok(Some(entry)) = extracted.next_entry().await {
        let file_name = entry.file_name().into_string().unwrap();
        for i in 29..=33 {
            if file_name.starts_with(&format!("x-{:>03}", i)) {
                continue 'entries;
            }
        }
        good_images.push(entry.path());
    }
    let good_images = good_images.iter().map(|n| n.as_path());
    shift_offset_cards(good_images, &path.join("cut"), 0, 66, 63).await;
}

async fn shift_offset_cards(
    images: impl Iterator<Item = &Path>,
    to: &Path,
    skip: usize,
    shift: usize,
    length: usize,
) {
    if matches!(fs::try_exists(&to).await, Ok(true)) {
        println!("{:?} cutouts already generated, skipping", to);
        return;
    }
    fs::create_dir(&to).await.unwrap();
    for (i, image) in images.enumerate() {
        if i < skip {
            fs::copy(
                image,
                to.join(format!("c-{:>03}.png", length - skip + i + shift)),
            )
            .await
            .unwrap();
            continue;
        }
        fs::copy(image, to.join(format!("c-{:>03}.png", i - skip + shift)))
            .await
            .unwrap();
    }
}

async fn download_set_pdf(url: &str, path: &Path) -> PathBuf {
    fs::create_dir_all(path).await.unwrap();
    let path = path.join("set.pdf");
    if matches!(fs::try_exists(&path).await, Ok(true)) {
        println!("{:?} already downloaded, skipping", path);
        return path;
    }
    let sg = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    fs::write(&path, sg).await.unwrap();
    path
}

async fn extract_images(from: &Path, to: &Path) -> fs::ReadDir {
    if matches!(fs::try_exists(&to).await, Ok(true)) {
        println!("{:?} already extracted, skipping", from);
        return fs::read_dir(to).await.unwrap();
    }
    fs::create_dir(&to).await.unwrap();
    tokio::process::Command::new("pdfimages")
        .arg("-png")
        .arg(from)
        .arg(&to.join("x"))
        .spawn()
        .unwrap()
        .wait()
        .await
        .unwrap();
    fs::read_dir(to).await.unwrap()
}

async fn crop_images(
    from: impl Iterator<Item = &Path>,
    to: &Path,
    width: u32,
    height: u32,
    top: u32,
    left: u32,
) -> fs::ReadDir {
    if matches!(fs::try_exists(&to).await, Ok(true)) {
        println!("{:?} crops already generated, skipping", to);
        return fs::read_dir(to).await.unwrap();
    }
    fs::create_dir_all(to).await.unwrap();

    for (i, image) in from.enumerate() {
        crop_raw(
            image,
            &to.join(format!("c-{:>03}.png", i)),
            width,
            height,
            left,
            top,
        )
        .await;
    }

    fs::read_dir(to).await.unwrap()
}

async fn cutout_images(from: impl Iterator<Item = &Path>, to: &Path, cutmap: &[[u32; 4]]) {
    if matches!(fs::try_exists(&to).await, Ok(true)) {
        println!("{:?} cutouts already generated, skipping", to);
        return;
    }
    fs::create_dir(&to).await.unwrap();

    let mut index = 1;
    for image in from {
        for cutout in cutmap {
            let [width, height, left, top] = *cutout;
            crop_raw(
                image,
                &to.join(format!("c-{:>03}.png", index)),
                width,
                height,
                left,
                top,
            )
            .await;
            index += 1;
        }
    }
}

async fn crop_raw(from: &Path, to: &Path, width: u32, height: u32, left: u32, top: u32) {
    tokio::process::Command::new("magick")
        .arg("convert")
        .arg(from)
        .arg("-crop")
        .arg(format!("{}x{}+{}+{}", width, height, left, top))
        .arg("+repage")
        .arg(to)
        .spawn()
        .unwrap()
        .wait()
        .await
        .unwrap();
}
