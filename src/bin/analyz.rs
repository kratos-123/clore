use indexmap::IndexMap;
use regex::Regex;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    time::Instant,
};

#[tokio::main]
async fn main() {
    let file = File::open("example/monitor.txt").await.unwrap();
    let address = "monitor";
    let reader = BufReader::new(file);
    let mut hashstring = IndexMap::<String, String>::new();
    let complex_regex = Regex::new(
            r"(Generating|Downloading|Map)([\w ]*:)[\t ]+([\d\.]+\%)\|[\S ]+\|[ ]+([\d]+)\/([\d]+)[ +][\[\S]+[ ]+([\d\.]+)[\w<? \w\/\]]+",
        )
        .unwrap();

    let bit_reg =
        Regex::new(r"([\d]+%)\|[\S ]+\|[ ]+([\d]+)\/([\d]+)[ +][\[\S]+[ ]+([\d\.]+)[ \w\/\]<?]+")
            .unwrap();

    let verify = Regex::new(r"\{'(loss|eval_loss).*}").unwrap();

    let complated = Regex::new(r"completed the task.*").unwrap();
    let mut instant = tokio::time::Instant::now();
    let mut lines = reader.lines();
    while let Ok(some_line) = lines.next_line().await {
        if let Some(line) = some_line {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if verify.captures(&line).is_some() {
                continue;
            }
            let complex = complex_regex.captures(&line);
            if complex.is_some() {
                let captures = complex.unwrap();
                let (_, [operate, extra, percent, prce, total, downspeed]) =
                    captures.extract::<6>();
                let string = format!(
                    "当前操作:{}{},完成百分比:{},完成进度:{}/{} 下载速度:{}",
                    operate, extra, percent, prce, total, downspeed
                );
                hashstring.insert(format!("{}{}", operate, extra), string);
                continue;
            }

            let bittest = bit_reg.captures(&line);
            if bittest.is_some() {
                let captures = bittest.unwrap();
                let (_, [percent, prce, total, it]) = captures.extract::<4>();
                let it = it.parse::<f32>().unwrap_or_default();
                let string = format!("{} {} {} {}", percent, prce, total, it);
                // println!("s:{}",string);
                // 验算时，这个算力的值非常大，不应该算进到日志里面去
                if it > 35f32 {
                    continue;
                }

                hashstring.insert("task_prcess".to_string(), string);
                continue;
            }

            if complated.captures(&line).is_some() {
                hashstring.insert("task compalte".to_string(), line.to_string());

                continue;
            }

            let string = format!("{} {}", address, line);
            hashstring.insert(line.to_string(), string);
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // println!("111");
        if !hashstring.is_empty() && instant.elapsed() > tokio::time::Duration::from_secs(10) {
            for (_, line) in hashstring.iter() {
                println!("{}", line);
            }
            instant = Instant::now();
            hashstring.clear();
        }
    }
}
