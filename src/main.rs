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
        //if number of guesses is more than 5 then game over
        if self.guesses.len() >= 5 {
            return Some(GameResult::Lose);
        }
        None
    }

    fn get_color_for_ltr(&self, ltr: &char) -> Color {
        //loop through each letter of the guesses
        for guess in &self.guesses {
            //for each letter of guess
            for (i, c) in guess.chars().enumerate() {
                //if the letter is the same as the letter we are checking
                if c == *ltr {
                    //if the letter is in the same position as the letter we are checking
                    if self.word.chars().nth(i).unwrap() == *ltr {
                        return Color::Green;
                    }
                    //if the letter is in the word but not in the same position
                    if self.word.contains(*ltr) {
                        return Color::Yellow;
                    }
                    //else return red
                    return Color::Red;
                }
            }
        }
        Color::Green
    }

    fn display_guesed_letters(&self) -> () {
        //get the guessed letters and find the color
        //initialize empty vec of colored strings
        let mut colored_letters: Vec<ColoredString> = vec![];
        //go through each guessed letter
        for ltr in self.guessed_letters.iter() {
            //get the color for the letter
            let color = self.get_color_for_ltr(ltr);
            //match color and push color to vec
            match color {
                Color::Green => colored_letters.push(ltr.to_string().green()),
                Color::Yellow => colored_letters.push(ltr.to_string().yellow()),
                Color::Red => colored_letters.push(ltr.to_string().red()),
            }
        }
        //print the vec
        //create an empty string with capacity of length of colored_letters
        let mut s = String::with_capacity(colored_letters.len());
        //for each colored letter push {} into string
        for _ in 0..colored_letters.len() {
            s.push_str("{}");
        }
        s = s.format(&colored_letters);
        println!("Guessed letters {}", s);
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
