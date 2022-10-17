use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use colored::{ColoredString, Colorize};
use dyn_fmt::AsStrFormatExt;
use rand::{random, thread_rng, Rng};

const WORD_SIZE: usize = 5;

pub struct GameStruct {
    word: String,
    guessed_letters: HashSet<char>,
    guesses: Vec<String>,
    dict: Vec<String>,
}

impl GameStruct {
    async fn new() -> Result<GameStruct, reqwest::Error> {
        //get the dict words.
        let handle = tokio::spawn(async move {
            let words = reqwest::Client::new()
                .get("https://www.wordgamedictionary.com/twl06/download/twl06.txt")
                .send()
                .await?
                .text()
                .await;
            words
        });
        let words = handle.await.unwrap();
        let a: Vec<String> = words
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>()
            .into_iter()
            .map(clean_str)
            .filter(|word| word.len() == 5)
            .collect();
        let word = &a[rand_bewteen(0, a.len(), None)];
        let dict = a.to_vec();
        Ok(GameStruct {
            word: word.to_owned(),
            guessed_letters: HashSet::new(),
            guesses: vec![],
            dict,
        })
    }

    fn ask_for_guess(&mut self) -> String {
        //loop until valid guess is true
        loop {
            let mut guess = "".to_string();
            println!("Enter your guess. Hint: {}", self.word);
            std::io::stdin().read_line(&mut guess).unwrap();
            guess = clean_str(&guess);
            //check valid guess
            if guess.len() == 5
                && guess.chars().all(|l| l.is_alphanumeric())
                && self.dict.contains(&guess)
            {
                self.guesses.push(guess.to_owned());
                //add the guessed letters
                for c in guess.chars() {
                    self.guessed_letters.insert(c);
                }
                return guess;
            }
            println!("not a valid guess!")
        }
    }

    fn is_game_over(&self, guess: &str) -> Option<GameResult> {
        if guess == self.word {
            return Some(GameResult::Win);
        }
        if self.guesses.len() > 4 {
            return Some(GameResult::Lose);
        }
        None
    }

    fn display_guesed_letters(&self) -> () {
        let mut guessed_letters = self
            .guessed_letters
            .clone()
            .into_iter()
            .collect::<Vec<char>>();
        guessed_letters.sort_by(|a, b| a.to_lowercase().cmp(b.to_lowercase()));
        let guessedstr: String = guessed_letters.iter().collect();
        let mut guessvalues: Vec<ColoredString> = vec![];
        let mut guessvaluesltr: Vec<GuessValues> = vec![];
        for c in guessedstr.chars() {
            for guess in self.guesses.to_vec().iter().rev() {
                for (pos, guess_ltr) in guess.chars().enumerate() {
                    let correct_ltr_at_idx = self.word.chars().nth(pos).unwrap();
                    //we need to overwrite if its red or yellow. if its green leave it
                    if c == guess_ltr {
                        //check for dupes based on letter
                        if guess_ltr == correct_ltr_at_idx {
                            //replace dont push
                            let idx = guessvaluesltr
                                .iter()
                                .position(|o| o.color == Color::Green || o.letter == c.to_string());
                            if let Some(idx) = idx {
                                guessvalues.push(format!("{}", c).green());
                                guessvaluesltr[idx] = GuessValues {
                                    letter: c.to_string(),
                                    color: Color::Green,
                                };
                            } else {
                                guessvalues.push(format!("{}", c).green());
                                guessvaluesltr.push(GuessValues {
                                    letter: c.to_string(),
                                    color: Color::Green,
                                });
                            }
                        } else if self.word.contains(guess_ltr) {
                            if !guessvaluesltr.iter().any(|o| o.has_ltr(c.to_string())) {
                                guessvalues.push(format!("{}", c).yellow());
                                guessvaluesltr.push(GuessValues {
                                    letter: c.to_string(),
                                    color: Color::Yellow,
                                });
                            }
                        } else {
                            if !guessvaluesltr.iter().any(|o| o.has_ltr(c.to_string())) {
                                guessvalues.push(format!("{}", c).red());
                                guessvaluesltr.push(GuessValues {
                                    letter: c.to_string(),
                                    color: Color::Red,
                                });
                            }
                        }
                    }
                }
            }
        }

        let mut new_str = "".to_string();
        for _ in 0..guessvalues.len() {
            new_str.push_str("{}");
        }
        new_str = new_str.format(&guessvalues);
        println!("Guessed values: {}", new_str);
    }

    fn display_guesses(&self) -> () {
        //display guessed letters and color them
        //loop through all guesses and print with colors
        for word in self.guesses.to_vec().iter() {
            let mut values: Vec<ColoredString> = vec![];
            for (ltr_i, ltr) in word.chars().into_iter().enumerate() {
                let correct_word_ltr_at_idx = self.word.chars().collect::<Vec<char>>()[ltr_i];
                if ltr == correct_word_ltr_at_idx {
                    values.push(format!("{}", ltr).green());
                } else if self.word.contains(ltr) {
                    values.push(format!("{}", ltr).yellow());
                } else {
                    values.push(format!("{}", ltr).red());
                }
            }
            let wordstr = "{}{}{}{}{}".format(&values) as String;
            println!("{}", wordstr);
        }
        self.display_guesed_letters();
    }
}

pub struct GuessValues {
    letter: String,
    color: Color,
}

impl GuessValues {
    fn has_ltr(&self, ltr: String) -> bool {
        self.letter == ltr
    }
    fn is_not_green(&self, ltr: String) -> bool {
        self.has_ltr(ltr) && self.color != Color::Green
    }
}

#[derive(PartialEq)]
pub enum Color {
    Green,
    Yellow,
    Red,
}

pub enum GameResult {
    Win,
    Lose,
}

fn rand_bewteen(start: usize, end: usize, exclude: Option<usize>) -> usize {
    let mut rng = rand::thread_rng();
    let mut attempt = rng.gen_range(start..end);
    if exclude.is_none() {
        return attempt;
    }
    let exclude = exclude.unwrap();
    loop {
        if attempt != exclude {
            return attempt;
        }
        attempt = rng.gen_range(start..end);
    }
}

#[tokio::main]
async fn main() {
    let mut game = GameStruct::new().await.unwrap();
    loop {
        let guess = game.ask_for_guess();
        game.display_guesses();
        if game.is_game_over(&guess).is_some() {
            let result = game.is_game_over(&guess).unwrap();
            match result {
                GameResult::Win => {
                    return println!("You win!");
                }
                GameResult::Lose => {
                    return println!("You lose!");
                }
                _ => unreachable!(),
            }
        }
    }
}

pub fn clean_str(str: &str) -> String {
    str.trim()
        .to_uppercase()
        .chars()
        .filter(|l| l.is_ascii_alphabetic())
        .collect()
}
