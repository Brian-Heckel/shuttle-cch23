use axum::{http::StatusCode, response::IntoResponse, Json};
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct NiceInput {
    input: String,
}

pub struct Naughty;

impl IntoResponse for Naughty {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::BAD_REQUEST,
            "{\"result\": \"naughty\"}".to_string(),
        )
            .into_response()
    }
}

/// Nice Strings must contain at least three vowels
/// aeioy
///
/// at least one letter that appears twice in a row and
/// must not contain the substrings ab, cd, pq, xy
#[tracing::instrument]
pub async fn nice(Json(nice_input): Json<NiceInput>) -> Result<Json<Value>, Naughty> {
    let contain_three_vowels =
        Regex::new(r"[aeiouyAEIOUY].*[aeiouyAEIOUY].*[aeiouyAEIOUY]").unwrap();
    let twice_one_letter = Regex::new(r"([a-zA-Z])\1").unwrap();
    let substrings = [
        Regex::new(r"ab").unwrap(),
        Regex::new(r"cd").unwrap(),
        Regex::new(r"pq").unwrap(),
        Regex::new(r"xy").unwrap(),
    ];

    let input = nice_input.input;
    let mut cond = true;
    cond = cond && contain_three_vowels.is_match(&input).unwrap();
    cond = cond && twice_one_letter.is_match(&input).unwrap();

    let is_valid = substrings
        .iter()
        .fold(cond, |c, regex| c && !(regex.is_match(&input).unwrap()));

    if is_valid {
        Ok(Json(json!({"result":"nice"})))
    } else {
        Err(Naughty)
    }
}

pub enum GameError {
    AtLeast8Long,
    ContainUpLowerDigits,
    Contain5Digits,
    AddTo2023,
    ContainJoyInOrder,
    HasSandwich,
    InRange,
    ContainEmoji,
    HashEndsWithA,
}

impl IntoResponse for GameError {
    fn into_response(self) -> axum::response::Response {
        match self {
            GameError::AtLeast8Long => {
                let response = Json(json!({"result":"naughty", "reason":"8 chars"}));
                (StatusCode::BAD_REQUEST, response).into_response()
            }
            GameError::ContainUpLowerDigits => {
                let response = Json(json!({"result":"naughty", "reason":"more types of chars"}));
                (StatusCode::BAD_REQUEST, response).into_response()
            }
            GameError::Contain5Digits => {
                let response = Json(json!({"result":"naughty", "reason":"55555"}));
                (StatusCode::BAD_REQUEST, response).into_response()
            }
            GameError::AddTo2023 => {
                let response = Json(json!({"result":"naughty", "reason":"math is hard"}));
                (StatusCode::BAD_REQUEST, response).into_response()
            }
            GameError::ContainJoyInOrder => {
                let response = Json(json!({"result":"naughty", "reason":"not joyful enough"}));
                (StatusCode::NOT_ACCEPTABLE, response).into_response()
            }
            GameError::HasSandwich => {
                let response = Json(json!({"result":"naughty", "reason":"illegal: no sandwich"}));
                (StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS, response).into_response()
            }
            GameError::InRange => {
                let response = Json(json!({"result":"naughty", "reason":"outranged"}));
                (StatusCode::RANGE_NOT_SATISFIABLE, response).into_response()
            }
            GameError::ContainEmoji => {
                let response = Json(json!({"result":"naughty", "reason":"😳"}));
                (StatusCode::UPGRADE_REQUIRED, response).into_response()
            }
            GameError::HashEndsWithA => {
                let response = Json(json!({"result":"naughty", "reason":"not a coffee brewer"}));
                (StatusCode::IM_A_TEAPOT, response).into_response()
            }
        }
    }
}

fn length_test(input: &str) -> Result<(), GameError> {
    if input.len() < 8 {
        Err(GameError::AtLeast8Long)
    } else {
        Ok(())
    }
}

fn case_test(input: &str) -> Result<(), GameError> {
    input
        .find(|c: char| c.is_uppercase())
        .ok_or(GameError::ContainUpLowerDigits)?;

    input
        .find(|c: char| c.is_lowercase())
        .ok_or(GameError::ContainUpLowerDigits)?;

    input
        .find(|c: char| c.is_ascii_digit())
        .ok_or(GameError::ContainUpLowerDigits)?;
    Ok(())
}

fn contain_five_digits(input: &str) -> Result<(), GameError> {
    let num_digits = input.chars().filter(|c| c.is_ascii_digit()).count();
    if num_digits > 4 {
        Ok(())
    } else {
        Err(GameError::Contain5Digits)
    }
}

fn adds_up(input: &str) -> Result<(), GameError> {
    let total = input
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u32>().unwrap())
        .sum::<u32>();
    if total == 2023 {
        Ok(())
    } else {
        Err(GameError::AddTo2023)
    }
}

fn is_joyful(input: &str) -> Result<(), GameError> {
    let regex = Regex::new(r"j.+?o.+?y").unwrap();
    if regex.is_match(input).unwrap() {
        Ok(())
    } else {
        Err(GameError::ContainJoyInOrder)
    }
}

fn has_sandwich(input: &str) -> Result<(), GameError> {
    let regex = Regex::new(r"([a-zA-Z])(?!\1).\1").unwrap();
    if regex.is_match(input).unwrap() {
        Ok(())
    } else {
        Err(GameError::HasSandwich)
    }
}

fn in_range(input: &str) -> Result<(), GameError> {
    let cond = input
        .chars()
        .any(|c| (&'\u{2980}'..=&'\u{2bFF}').contains(&&c));
    if cond {
        Ok(())
    } else {
        Err(GameError::InRange)
    }
}

fn has_emoji(input: &str) -> Result<(), GameError> {
    let cond = emojis::iter().any(|e| input.contains(e.as_str()));
    if cond {
        Ok(())
    } else {
        Err(GameError::ContainEmoji)
    }
}

fn check_hash(input: &str) -> Result<(), GameError> {
    let cond = sha256::digest(input).ends_with("a");
    if cond {
        Ok(())
    } else {
        Err(GameError::HashEndsWithA)
    }
}

#[tracing::instrument]
pub async fn game(Json(nice_input): Json<NiceInput>) -> Result<Json<Value>, GameError> {
    let input = nice_input.input;
    length_test(&input)?;
    case_test(&input)?;
    contain_five_digits(&input)?;
    adds_up(&input)?;
    is_joyful(&input)?;
    has_sandwich(&input)?;
    in_range(&input)?;
    has_emoji(&input)?;
    check_hash(&input)?;
    let response = Json(json!({"result":"nice","reason":"that's a nice password"}));
    Ok(response)
}
