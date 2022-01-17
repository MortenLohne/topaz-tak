use crate::eval::Evaluator6;
use anyhow::Result;
use board_game_traits::Position;
use crossbeam_channel::{unbounded, Receiver, Sender};
use getopts::Options;
use std::env;
use std::io::{self, BufRead};
use std::thread;
use std::time::Instant;
use topaz_tak::board::Board6;
use topaz_tak::eval::Weights6;
use topaz_tak::search::{proof::TinueSearch, search, SearchInfo};
use topaz_tak::*;

pub fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(arg1) = args.get(1) {
        if arg1 == "black" {
            play_game_cmd(false);
        } else if arg1 == "white" {
            play_game_cmd(true);
        } else if arg1 == "test" {
            let time = Instant::now();
            let s = "2,x4,1/x4,1,x/x,2,12C,1,1,x/x,1,2,21C,x2/x,2,2,x3/x2,2,1,x2 1 10";
            let mut board = Board6::try_from_tps(s).unwrap();
            let eval = Evaluator6 {};
            let mut info = SearchInfo::new(6, 10000);
            search(&mut board, &eval, &mut info);
            let pv_move = info.pv_move(&board).unwrap();
            println!("Computer Choose: {}", pv_move.to_ptn::<Board6>());
            println!("Time: {} ms", time.elapsed().as_millis());
            return;
        } else if arg1 == "tinue" {
            let mut rest = String::new();
            for s in args[2..].iter() {
                rest.push_str(s);
                rest.push_str(" ");
            }
            rest.pop();
            let tps = match rest.as_str() {
                "alion1" => "2,1221122,1,1,1,2S/1,1,1,x,1C,1111212/x2,2,212,2C,11/2,2,x2,1,1/x3,1,1,x/x2,2,21,x,112S 2 32",
                "alion2" => "2,212221C,2,2,2C,1/1,2,1,1,2,1/12,x,1S,2S,2,1/2,2,2,x2,1/1,2212121S,2,12,1,1S/x,2,2,2,x,1 1 30",
                "alion3" => "x2,1,21,2,2/1,2,21,1,21,2/1S,2,2,2C,2,2/21S,1,121C,x,1,12/2,2,121,1,1,1/2,2,x3,22S 1 27",
                "alion4" => "x,1,x4/2,2,1,1,1,1/2221,x,1,21C,x2/2,2,2C,1,2,x/2,2,1,1,1,2/2,x2,2,x,1 2 18",
                "alion5" => "2,x4,11/x5,221/x,2,2,2,x,221/2,1,12C,1,21C,2/2,x,2,x2,2/x,2,2,2,x,121 1 25",
                "test5" => "2,2,x2,1/2,2,x,1,1/1221S,1,122221C,x,1/1,12,x,2C,2/1S,2,2,x2 1 20",
                "test7" => concat!("2,2,21S,2,1,1,1/2,1,x,2,1,x,1/2,2,2,2,21112C,121S,x/x2,1112C,2,1,1112S,x/121,22211C,", 
                    "1S,1,1,121,1221C/x,2,2,2,1,12,2/2,x3,1,122,x 2 50"),
                _ => rest.as_str(),
            };
            let game = match TakGame::try_from_tps(tps) {
                Ok(b) => b,
                Err(e) => {
                    println!("Unable to create game with tps: \n{}\n{}", tps, e);
                    return;
                }
            };
            match game {
                TakGame::Standard5(board) => {
                    let search = crate::search::proof::TinueSearch::new(board);
                    proof_interactive(search).unwrap();
                }
                TakGame::Standard6(board) => {
                    let search = crate::search::proof::TinueSearch::new(board);
                    proof_interactive(search).unwrap();
                }
                TakGame::Standard7(board) => {
                    let search = crate::search::proof::TinueSearch::new(board);
                    proof_interactive(search).unwrap();
                }
                _ => todo!(),
            }
            return;
        } else {
            println!("Unknown argument: {}", arg1);
        }
    }
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Could not read line");
    if buffer.trim() == "tei" {
        let (s, r) = unbounded();
        tei_loop(s);
        identify();
        let _ = play_game_tei(r);
    } else if buffer == "play white" {
        play_game_cmd(true)
    } else if buffer == "play black" {
        play_game_cmd(false)
    } else {
        println!("Unknown command: {}", buffer);
    }
}

fn proof_interactive<T: TakBoard>(mut search: TinueSearch<T>) -> Result<()> {
    let time = Instant::now();
    let tinue = search.is_tinue().unwrap();
    if tinue {
        println!("Tinue Found!")
    } else {
        println!("No Tinue Found.");
    }
    let pv = search.principal_variation();
    for m in pv.into_iter().map(|m| m.to_ptn::<T>()) {
        println!("{}", m);
    }

    let seconds = time.elapsed().as_secs();
    println!("Done in {} seconds", seconds);
    let mut interactive = crate::search::proof::InteractiveSearch::new(search);
    let mut first = true;
    interactive.print_root();
    loop {
        let mut opts = Options::new();
        let mut buffer = String::new();
        // opts.optopt("o", "", "set output file name", "NAME");
        opts.optopt("m", "move", "Move the root of the tree", "PTN/PTN");
        opts.optopt(
            "e",
            "expand",
            "Expand the tree of a certain move",
            "PTN/PTN",
        );
        // opts.optflag("v", "verbose", "Expand all children, even explored ones");
        opts.optflag("h", "help", "Print the help text");
        opts.optflag("q", "quit", "Quit");
        opts.optflag(
            "r",
            "reset",
            "Resets the view back to the default root view",
        );
        if first {
            println!("{}", opts.usage(""));
            first = false;
        }
        io::stdin().lock().read_line(&mut buffer)?;
        let matches = opts.parse(buffer.split_whitespace())?;
        if matches.opt_present("q") {
            break;
        }
        if matches.opt_present("h") {
            println!("{}", opts.usage(""));
            continue;
        }
        if matches.opt_present("r") {
            interactive.reset_expansion();
            interactive.reset_view();
        }
        if let Some(v) = matches.opt_str("m") {
            let res = interactive.change_view(&v);
            if res.is_err() {
                println!("Failed to change view, resetting to default!");
                interactive.reset_view();
            }
        }
        if let Some(s) = matches.opt_str("e") {
            interactive.expand_line(s.split("/").collect());
        }
        // interactive.expand_line(vec!["c1".to_string(), "b1>".to_string()]);
        interactive.print_root();
    }
    Ok(())
}

fn play_game_cmd(mut computer_turn: bool) {
    let mut board = Board6::new();
    let eval = Evaluator6 {};
    while let None = board.game_result() {
        println!("{:?}", &board);
        if computer_turn {
            let mut info = SearchInfo::new(6, 5000);
            search(&mut board, &eval, &mut info);
            let pv_move = info.pv_move(&board).unwrap();
            println!("Computer Choose: {}", pv_move.to_ptn::<Board6>());
            board.do_move(pv_move);
        } else {
            let stdin = io::stdin();
            let line = stdin.lock().lines().next().unwrap().unwrap();
            if line == "q" {
                return;
            } else if line == "tps" {
                println!("{:?}", &board);
                continue;
            }
            let ptn_move = GameMove::try_from_ptn(&line, &board).unwrap();
            if !board.legal_move(ptn_move) {
                println!("Illegal Move Attempted!");
                continue;
            }
            board.do_move(ptn_move);
        }
        computer_turn = !computer_turn;
    }
}

struct TimeLeft {
    wtime: u64,
    btime: u64,
    winc: u64,
    binc: u64,
}

impl TimeLeft {
    pub fn new(tei_str: &str) -> Self {
        let mut ret = Self {
            wtime: 1000,
            btime: 1000,
            winc: 0,
            binc: 0,
        };
        for (field, val) in tei_str
            .split_whitespace()
            .zip(tei_str.split_whitespace().skip(1))
        {
            match (field, val.parse()) {
                ("wtime", Ok(val)) => ret.wtime = val,
                ("btime", Ok(val)) => ret.btime = val,
                ("winc", Ok(val)) => ret.winc = val,
                ("binc", Ok(val)) => ret.binc = val,
                _ => {}
            }
        }
        ret
    }
    fn use_time(&self, est_plies: usize, side_to_move: Color) -> u64 {
        let (time_bank, inc) = match side_to_move {
            Color::White => (self.wtime, self.winc),
            Color::Black => (self.btime, self.binc),
        };
        let use_bank = time_bank / (est_plies + 2) as u64 / 1000;
        use_bank + inc / 1000
    }
}

fn play_game_tei(receiver: Receiver<TeiCommand>) -> Result<()> {
    let mut board = Board6::new();
    let mut info = SearchInfo::new(6, 1000000);
    let mut eval = Weights6::default();
    eval.add_noise();
    // let eval = Evaluator6 {};
    loop {
        let message = receiver.recv()?;
        match message {
            TeiCommand::Go(s) => {
                let low_flats = std::cmp::min(
                    board.pieces_reserve(Color::White),
                    board.pieces_reserve(Color::Black),
                );
                let est_plies = low_flats * 2;
                let time_left = TimeLeft::new(&s);
                let use_time = time_left.use_time(est_plies, board.side_to_move());
                info = SearchInfo::new(6, 0)
                    .take_table(&mut info)
                    .max_time(use_time);
                let res = search(&mut board, &eval, &mut info);
                if let Some(outcome) = res {
                    println!("info {}", outcome);
                    println!(
                        "bestmove {}",
                        outcome
                            .best_move()
                            .expect("Could not find best move!")
                            .trim_end_matches('*')
                    );
                } else {
                    println!("Something went wrong, search failed!");
                }
            }
            TeiCommand::Position(s) => {
                board = Board6::new();
                for m in s.split_whitespace() {
                    if let Some(m) = GameMove::try_from_ptn(m, &board) {
                        board.do_move(m);
                    }
                }
            }
            TeiCommand::Quit => {
                break;
            }
            _ => println!("Unknown command: {:?}", message),
        }
    }
    Ok(())
}

fn identify() {
    println!("id name Topaz");
    println!("id author Justin Kur");
    println!("teiok");
}

fn tei_loop(sender: Sender<TeiCommand>) {
    thread::spawn(move || {
        let mut buffer = String::new();
        loop {
            std::io::stdin()
                .read_line(&mut buffer)
                .expect("Could not read line");
            let line = buffer.trim();
            if line == "tei" {
                identify();
            } else if line == "isready" {
                println!("readyok");
            } else if line == "quit" {
                sender.send(TeiCommand::Quit).unwrap();
                break;
            } else if line == "stop" {
                sender.send(TeiCommand::Stop).unwrap();
            } else if line.starts_with("position") {
                sender.send(TeiCommand::Position(line.to_string())).unwrap();
            } else if line.starts_with("go") {
                sender.send(TeiCommand::Go(line.to_string())).unwrap();
            } else {
                println!("Unknown Tei Command: {}", buffer);
            }
            buffer.clear();
        }
    });
}
